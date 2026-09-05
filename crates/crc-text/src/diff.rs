#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    Same { text: String, before: usize, after: usize },
    Added { text: String, after: usize },
    Removed { text: String, before: usize },
    Skipped(usize),
}

impl Line {
    pub fn text(&self) -> &str {
        match self {
            Line::Same { text, .. } | Line::Added { text, .. } | Line::Removed { text, .. } => {
                text
            }
            Line::Skipped(_) => "",
        }
    }

    pub fn marker(&self) -> char {
        match self {
            Line::Same { .. } => ' ',
            Line::Added { .. } => '+',
            Line::Removed { .. } => '-',
            Line::Skipped(_) => '~',
        }
    }
}

pub const TOO_BIG: usize = 1200;

pub fn lines(before: &str, after: &str) -> Vec<Line> {
    let old: Vec<&str> = before.lines().collect();
    let new: Vec<&str> = after.lines().collect();

    let head = old
        .iter()
        .zip(new.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let tail = old[head..]
        .iter()
        .rev()
        .zip(new[head..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();

    let mut changes = Vec::new();
    for (index, line) in old.iter().take(head).enumerate() {
        changes.push(Line::Same {
            text: line.to_string(),
            before: index,
            after: index,
        });
    }

    let old_middle = &old[head..old.len() - tail];
    let new_middle = &new[head..new.len() - tail];

    if old_middle.len().max(new_middle.len()) > TOO_BIG {
        for (offset, line) in old_middle.iter().enumerate() {
            changes.push(Line::Removed {
                text: line.to_string(),
                before: head + offset,
            });
        }
        for (offset, line) in new_middle.iter().enumerate() {
            changes.push(Line::Added {
                text: line.to_string(),
                after: head + offset,
            });
        }
    } else {
        changes.extend(walk(old_middle, new_middle, head));
    }

    for offset in 0..tail {
        changes.push(Line::Same {
            text: old[old.len() - tail + offset].to_string(),
            before: old.len() - tail + offset,
            after: new.len() - tail + offset,
        });
    }

    changes
}

fn walk(old: &[&str], new: &[&str], offset: usize) -> Vec<Line> {
    let rows = old.len();
    let columns = new.len();

    let mut table = vec![0usize; (rows + 1) * (columns + 1)];
    let at = |row: usize, column: usize| row * (columns + 1) + column;

    for row in (0..rows).rev() {
        for column in (0..columns).rev() {
            table[at(row, column)] = if old[row] == new[column] {
                table[at(row + 1, column + 1)] + 1
            } else {
                table[at(row + 1, column)].max(table[at(row, column + 1)])
            };
        }
    }

    let mut changes = Vec::new();
    let (mut row, mut column) = (0usize, 0usize);

    while row < rows && column < columns {
        if old[row] == new[column] {
            changes.push(Line::Same {
                text: old[row].to_string(),
                before: offset + row,
                after: offset + column,
            });
            row += 1;
            column += 1;
        } else if table[at(row + 1, column)] >= table[at(row, column + 1)] {
            changes.push(Line::Removed {
                text: old[row].to_string(),
                before: offset + row,
            });
            row += 1;
        } else {
            changes.push(Line::Added {
                text: new[column].to_string(),
                after: offset + column,
            });
            column += 1;
        }
    }

    while row < rows {
        changes.push(Line::Removed {
            text: old[row].to_string(),
            before: offset + row,
        });
        row += 1;
    }
    while column < columns {
        changes.push(Line::Added {
            text: new[column].to_string(),
            after: offset + column,
        });
        column += 1;
    }

    changes
}

pub fn around(changes: Vec<Line>, context: usize) -> Vec<Line> {
    let interesting: Vec<bool> = changes
        .iter()
        .map(|change| !matches!(change, Line::Same { .. }))
        .collect();

    if !interesting.iter().any(|found| *found) {
        return Vec::new();
    }

    let mut keep = vec![false; changes.len()];
    for (index, found) in interesting.iter().enumerate() {
        if !found {
            continue;
        }
        let from = index.saturating_sub(context);
        let to = (index + context + 1).min(changes.len());
        keep[from..to].iter_mut().for_each(|slot| *slot = true);
    }

    let mut out = Vec::new();
    let mut dropped = 0usize;

    for (index, change) in changes.into_iter().enumerate() {
        if keep[index] {
            if dropped > 0 {
                out.push(Line::Skipped(dropped));
                dropped = 0;
            }
            out.push(change);
        } else {
            dropped += 1;
        }
    }

    if dropped > 0 {
        out.push(Line::Skipped(dropped));
    }

    out
}

pub fn tally(changes: &[Line]) -> (usize, usize) {
    let added = changes
        .iter()
        .filter(|change| matches!(change, Line::Added { .. }))
        .count();
    let removed = changes
        .iter()
        .filter(|change| matches!(change, Line::Removed { .. }))
        .count();
    (added, removed)
}
