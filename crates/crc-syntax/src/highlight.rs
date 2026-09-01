use std::ops::Range;

use crc_theme::Highlight;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub range: Range<usize>,
    pub highlight: Highlight,
}

pub fn role_for(capture: &str) -> Highlight {
    match capture {
        c if c.starts_with("variable.parameter") => Highlight::Parameter,
        c if c.starts_with("string.special.key") => Highlight::Parameter,
        c if c.starts_with("keyword") => Highlight::Keyword,
        c if c.starts_with("comment") => Highlight::Comment,
        c if c.starts_with("string") || c.starts_with("character") => Highlight::String,
        c if c.starts_with("escape") => Highlight::String,
        c if c.starts_with("number") || c.starts_with("float") => Highlight::Number,
        c if c.starts_with("boolean") || c.starts_with("constant") => Highlight::Number,
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

pub fn resolve(mut captures: Vec<(Range<usize>, Highlight)>) -> Vec<HighlightSpan> {
    captures.sort_by(|a, b| a.0.start.cmp(&b.0.start).then(b.0.end.cmp(&a.0.end)));

    let mut spans: Vec<HighlightSpan> = Vec::with_capacity(captures.len());
    let mut stack: Vec<(usize, Highlight)> = Vec::new();
    let mut cursor = 0usize;

    let emit = |spans: &mut Vec<HighlightSpan>, range: Range<usize>, highlight: Highlight| {
        if range.is_empty() {
            return;
        }
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
        while let Some(&(end, top)) = stack.last() {
            if end > range.start {
                break;
            }
            emit(&mut spans, cursor..end, top);
            cursor = cursor.max(end);
            stack.pop();
        }

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
