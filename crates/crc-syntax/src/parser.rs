use std::ops::Range;

use streaming_iterator::StreamingIterator;
use tree_sitter::{InputEdit, Parser, Point, Query, QueryCursor, Tree};

use crate::error::{Result, SyntaxError};
use crate::fault::Fault;
use crate::highlight::{HighlightSpan, resolve, role_for};
use crate::language::Language;

pub struct SyntaxTree {
    language: Language,
    parser: Parser,
    query: Query,
    roles: Vec<crc_theme::Highlight>,
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

    pub fn has_error(&self) -> bool {
        self.tree
            .as_ref()
            .is_some_and(|t| t.root_node().has_error())
    }

    pub fn reset(&mut self) {
        self.tree = None;
    }

    pub fn parse(&mut self, text: &str) -> Result<()> {
        let tree = self
            .parser
            .parse(text, self.tree.as_ref())
            .ok_or(SyntaxError::Parse(self.language.name()))?;
        self.tree = Some(tree);
        Ok(())
    }

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

    pub fn faults(&self) -> Vec<Fault> {
        let Some(tree) = self.tree.as_ref() else {
            return Vec::new();
        };
        if !tree.root_node().has_error() {
            return Vec::new();
        }

        let mut faults = Vec::new();
        let mut cursor = tree.walk();
        let mut down = true;

        loop {
            if down {
                let node = cursor.node();
                if node.is_error() || node.is_missing() {
                    let start = node.start_position();
                    faults.push(Fault {
                        range: node.byte_range(),
                        line: start.row,
                        column: start.column,
                        missing: node.is_missing(),
                        kind: node.kind().to_string(),
                    });
                    down = false;
                } else if node.has_error() && cursor.goto_first_child() {
                    continue;
                } else {
                    down = false;
                }
            }

            if cursor.goto_next_sibling() {
                down = true;
                continue;
            }
            if !cursor.goto_parent() {
                break;
            }
        }

        faults.truncate(200);
        faults
    }

    pub fn highlights(&self, text: &str) -> Vec<HighlightSpan> {
        self.highlights_in(text, 0..text.len())
    }

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

struct Candidate {
    range: Range<usize>,
    role: crc_theme::Highlight,
    specificity: usize,
    pattern: usize,
}

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
