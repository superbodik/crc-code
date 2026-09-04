#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub text: String,
    pub foreground: Ink,
    pub background: Ink,
    pub bold: bool,
    pub inverse: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            text: " ".to_string(),
            foreground: Ink::Default,
            background: Ink::Default,
            bold: false,
            inverse: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Row {
    pub cells: Vec<Cell>,
}

impl Row {
    pub fn text(&self) -> String {
        self.cells.iter().map(|cell| cell.text.as_str()).collect()
    }

    pub fn is_blank(&self) -> bool {
        self.cells
            .iter()
            .all(|cell| cell.text.trim().is_empty() && cell.background == Ink::Default)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Screen {
    pub rows: Vec<Row>,
    pub cursor: (u16, u16),
    pub cursor_visible: bool,
    pub alive: bool,
}

impl Screen {
    pub fn size(&self) -> (usize, usize) {
        let columns = self.rows.first().map(|row| row.cells.len()).unwrap_or(0);
        (self.rows.len(), columns)
    }

    pub fn trimmed(&self) -> &[Row] {
        let last = self
            .rows
            .iter()
            .rposition(|row| !row.is_blank())
            .map(|index| index + 1)
            .unwrap_or(0);
        &self.rows[..last]
    }
}
