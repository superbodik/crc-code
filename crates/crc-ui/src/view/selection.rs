use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    pub row: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub to_line_end: bool,
}

pub fn bands(text: &str, selection: &Range<usize>) -> Vec<Band> {
    if selection.start >= selection.end {
        return Vec::new();
    }

    let mut bands = Vec::new();
    let mut offset = 0usize;

    for (row, line) in text.split_inclusive('\n').enumerate() {
        let stripped = line.trim_end_matches(['\n', '\r']);
        let content_end = offset + stripped.len();
        let line_end = offset + line.len();

        let start = selection.start.max(offset);
        let end = selection.end.min(line_end);
        if start >= end {
            offset = line_end;
            continue;
        }

        let start_column = column_of(stripped, start.saturating_sub(offset));
        let end_column = column_of(stripped, end.min(content_end).saturating_sub(offset));
        let to_line_end = end > content_end && line.len() > stripped.len();

        if end_column > start_column || to_line_end {
            bands.push(Band {
                row,
                start_column,
                end_column,
                to_line_end,
            });
        }
        offset = line_end;
    }

    bands
}

fn column_of(line: &str, byte: usize) -> usize {
    let byte = byte.min(line.len());
    let byte = (0..=byte)
        .rev()
        .find(|b| line.is_char_boundary(*b))
        .unwrap_or(0);
    line[..byte].chars().count()
}
