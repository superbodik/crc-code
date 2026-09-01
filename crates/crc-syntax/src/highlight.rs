use std::ops::Range;

use crc_theme::Highlight;

/// A run of bytes that share one highlight role.
///
/// Byte offsets, because that is what tree-sitter speaks. The buffer converts
/// to character offsets at the boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub range: Range<usize>,
    pub highlight: Highlight,
}

/// Map a tree-sitter capture name onto one of the theme's roles.
///
/// Capture names are dotted and open-ended — `keyword.control.conditional`,
/// `variable.parameter.builtin` — and every grammar invents its own. Matching
/// on the leading segments keeps one grammar's vocabulary from needing a new
/// arm here.
pub fn role_for(capture: &str) -> Highlight {
    // Longest-prefix first: `variable.parameter` is a parameter, bare
    // `variable` is not.
    match capture {
        c if c.starts_with("variable.parameter") => Highlight::Parameter,
        // Grammars capture object keys as `string.special.key`. They are keys
        // first and strings second, and reading JSON is much easier when they
        // do not blend into the values beside them.
        c if c.starts_with("string.special.key") => Highlight::Parameter,
        c if c.starts_with("keyword") => Highlight::Keyword,
        c if c.starts_with("comment") => Highlight::Comment,
        c if c.starts_with("string") || c.starts_with("character") => Highlight::String,
        c if c.starts_with("escape") => Highlight::String,
        c if c.starts_with("number") || c.starts_with("float") => Highlight::Number,
        c if c.starts_with("boolean") || c.starts_with("constant") => Highlight::Number,
        // Types, constructors and JSX tags share the blue that function names
        // use — the design colours `Editor`, `Surface` and `useEditor` alike.
        c if c.starts_with("function") || c.starts_with("method") => Highlight::Function,
        c if c.starts_with("type") || c.starts_with("constructor") => Highlight::Function,
        c if c.starts_with("tag") => Highlight::Function,
        c if c.starts_with("property") || c.starts_with("attribute") => Highlight::Parameter,
        c if c.starts_with("label") => Highlight::Keyword,
        c if c.starts_with("punctuation") || c.starts_with("operator") => Highlight::Punctuation,
        c if c.starts_with("delimiter") => Highlight::Punctuation,
        _ => Highlight::Text,
    }
}

/// Flatten overlapping captures into a run of spans that do not overlap.
///
/// Query captures nest — a whole call expression can be captured as well as
/// the identifier inside it — and the innermost one has to win for the bytes
/// it covers, while the outer one keeps the rest. Handled with a stack rather
/// than by dropping the outer capture, so nothing loses its colour.
///
/// `captures` may arrive in any order.
pub fn resolve(mut captures: Vec<(Range<usize>, Highlight)>) -> Vec<HighlightSpan> {
    // Outermost first at a shared start, so a parent is on the stack before
    // its child arrives.
    captures.sort_by(|a, b| a.0.start.cmp(&b.0.start).then(b.0.end.cmp(&a.0.end)));

    let mut spans: Vec<HighlightSpan> = Vec::with_capacity(captures.len());
    let mut stack: Vec<(usize, Highlight)> = Vec::new();
    let mut cursor = 0usize;

    let emit = |spans: &mut Vec<HighlightSpan>, range: Range<usize>, highlight: Highlight| {
        if range.is_empty() {
            return;
        }
        // Merge with the previous span when it is the same role and touches.
        if let Some(last) = spans.last_mut()
            && last.highlight == highlight
            && last.range.end == range.start
        {
            last.range.end = range.end;
            return;
        }
        spans.push(HighlightSpan { range, highlight });
    };

    for (range, highlight) in captures {
        // Close everything that ends before this capture begins.
        while let Some(&(end, top)) = stack.last() {
            if end > range.start {
                break;
            }
            emit(&mut spans, cursor..end, top);
            cursor = cursor.max(end);
            stack.pop();
        }

        // Whatever is still open covers the gap up to here.
        if let Some(&(_, top)) = stack.last() {
            emit(&mut spans, cursor..range.start, top);
        }
        cursor = cursor.max(range.start);
        stack.push((range.end, highlight));
    }

    while let Some((end, top)) = stack.pop() {
        emit(&mut spans, cursor..end, top);
        cursor = cursor.max(end);
    }

    spans
}
