use std::ops::Range;

use streaming_iterator::StreamingIterator;
use tree_sitter::{InputEdit, Parser, Point, Query, QueryCursor, Tree};

use crate::error::{Result, SyntaxError};
use crate::highlight::{HighlightSpan, resolve, role_for};
use crate::language::Language;

/// A parsed buffer, kept in step with its text.
///
/// The tree is reused across edits: tell it what changed with
/// [`edit`](SyntaxTree::edit) before re-parsing and tree-sitter walks only the
/// damaged part. That is the difference between highlighting that keeps up
/// with typing and highlighting that re-parses a 10k-line file per keystroke.
pub struct SyntaxTree {
    language: Language,
    parser: Parser,
    query: Query,
    /// Roles per capture index, resolved once instead of per match.
    roles: Vec<crc_theme::Highlight>,
    /// How qualified each capture name is: `string.special.key` is 3.
    specificity: Vec<usize>,
    tree: Option<Tree>,
}

impl SyntaxTree {
    pub fn new(language: Language) -> Result<Self> {
        let grammar = language.grammar();

        let mut parser = Parser::new();
        parser
            .set_language(&grammar)
            .map_err(|_| SyntaxError::Grammar(language.name()))?;

        let query = Query::new(&grammar, &language.highlights_query()).map_err(|source| {
            SyntaxError::Query {
                language: language.name(),
                source,
            }
        })?;

        let roles = query.capture_names().iter().map(|n| role_for(n)).collect();
        let specificity = query
            .capture_names()
            .iter()
            .map(|name| name.split('.').count())
            .collect();

        Ok(Self {
            language,
            parser,
            query,
            roles,
            specificity,
            tree: None,
        })
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn tree(&self) -> Option<&Tree> {
        self.tree.as_ref()
    }

    /// Whether the parse hit anything it could not make sense of. Normal while
    /// a line is half-typed; the tree is still usable.
    pub fn has_error(&self) -> bool {
        self.tree
            .as_ref()
            .is_some_and(|t| t.root_node().has_error())
    }

    /// Parse `text`, reusing the previous tree when [`edit`](SyntaxTree::edit)
    /// has described what changed.
    pub fn parse(&mut self, text: &str) -> Result<()> {
        let tree = self
            .parser
            .parse(text, self.tree.as_ref())
            .ok_or(SyntaxError::Parse(self.language.name()))?;
        self.tree = Some(tree);
        Ok(())
    }

    /// Tell the tree what changed, in byte offsets, before the next parse.
    ///
    /// `before` and `after` are the whole text on either side of the edit —
    /// tree-sitter wants line/column for each end, and only the text can say
    /// where those are.
    pub fn edit(
        &mut self,
        before: &str,
        after: &str,
        start: usize,
        old_end: usize,
        new_end: usize,
    ) {
        let Some(tree) = self.tree.as_mut() else {
            return;
        };
        tree.edit(&InputEdit {
            start_byte: start,
            old_end_byte: old_end,
            new_end_byte: new_end,
            start_position: point_at(before, start),
            old_end_position: point_at(before, old_end),
            new_end_position: point_at(after, new_end),
        });
    }

    /// Highlight the whole buffer.
    pub fn highlights(&self, text: &str) -> Vec<HighlightSpan> {
        self.highlights_in(text, 0..text.len())
    }

    /// Highlight one byte range — what the renderer asks for, since only the
    /// visible lines need colouring.
    pub fn highlights_in(&self, text: &str, range: Range<usize>) -> Vec<HighlightSpan> {
        let Some(tree) = self.tree.as_ref() else {
            return Vec::new();
        };

        let mut cursor = QueryCursor::new();
        cursor.set_byte_range(range);

        let mut captures = Vec::new();
        let mut matches = cursor.matches(&self.query, tree.root_node(), text.as_bytes());
        while let Some(m) = matches.next() {
            let pattern = m.pattern_index;
            for capture in m.captures() {
                let index = capture.index as usize;
                captures.push(Candidate {
                    range: capture.node.byte_range(),
                    role: self.roles[index],
                    specificity: self.specificity[index],
                    pattern,
                });
            }
        }

        // Two rules routinely claim the same node, and the grammars disagree
        // about which should win by position: the JavaScript query opens with
        // `(identifier) @variable` and refines it further down, while the JSON
        // query states `@string.special.key` before the plain `(string)`.
        // Neither "first wins" nor "last wins" satisfies both.
        //
        // The capture name settles it instead: the more qualified name is the
        // more specific claim, so `string.special.key` beats `string`. Only
        // when names are equally qualified does position decide, and there the
        // later rule is the refinement — which is what makes a capitalised
        // identifier a component rather than a plain variable.
        captures.sort_by(|a, b| {
            a.range
                .start
                .cmp(&b.range.start)
                .then(b.range.end.cmp(&a.range.end))
                .then(b.specificity.cmp(&a.specificity))
                .then(b.pattern.cmp(&a.pattern))
        });
        captures.dedup_by(|a, b| a.range == b.range);

        resolve(captures.into_iter().map(|c| (c.range, c.role)).collect())
    }
}

impl std::fmt::Debug for SyntaxTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyntaxTree")
            .field("language", &self.language)
            .field("parsed", &self.tree.is_some())
            .finish()
    }
}

/// One rule's claim on a node, before the claims are reconciled.
struct Candidate {
    range: Range<usize>,
    role: crc_theme::Highlight,
    specificity: usize,
    pattern: usize,
}

/// Line and column, in bytes, of an offset.
fn point_at(text: &str, offset: usize) -> Point {
    let offset = offset.min(text.len());
    let before = &text[..offset];
    let row = before.matches('\n').count();
    let column = match before.rfind('\n') {
        Some(newline) => offset - newline - 1,
        None => offset,
    };
    Point { row, column }
}
