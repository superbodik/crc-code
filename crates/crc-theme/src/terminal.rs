use crate::color::Rgba;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalTheme {
    pub background: Rgba,
    pub foreground: Rgba,
    pub cursor: Rgba,
    pub selection: Rgba,
    pub normal: [Rgba; 8],
    pub bright: [Rgba; 8],
}

impl TerminalTheme {
    pub fn colour(&self, index: u8) -> Rgba {
        match index {
            0..=7 => self.normal[index as usize],
            8..=15 => self.bright[(index - 8) as usize],
            16..=231 => {
                let step = |value: u8| -> u8 {
                    if value == 0 { 0 } else { 55 + value * 40 }
                };
                let n = index - 16;
                Rgba::new(step(n / 36), step((n / 6) % 6), step(n % 6), 255)
            }
            _ => {
                let level = 8 + (index - 232) * 10;
                Rgba::new(level, level, level, 255)
            }
        }
    }

    pub fn dark() -> Self {
        Self {
            background: Rgba::hex(0x11141a),
            foreground: Rgba::hex(0xd4d9e1),
            cursor: Rgba::hex(0x4aa8ff),
            selection: Rgba::hex(0x264264),
            normal: [
                Rgba::hex(0x3b4048),
                Rgba::hex(0xe06c75),
                Rgba::hex(0x8cc265),
                Rgba::hex(0xd6b871),
                Rgba::hex(0x61a8ff),
                Rgba::hex(0xc678dd),
                Rgba::hex(0x56b6c2),
                Rgba::hex(0xc0c6d0),
            ],
            bright: [
                Rgba::hex(0x5c6370),
                Rgba::hex(0xf08a92),
                Rgba::hex(0xa5d97f),
                Rgba::hex(0xe8cf8c),
                Rgba::hex(0x8fc2ff),
                Rgba::hex(0xd79bec),
                Rgba::hex(0x77ccd8),
                Rgba::hex(0xf0f3f8),
            ],
        }
    }

    pub fn light() -> Self {
        Self {
            background: Rgba::hex(0xfbfcfd),
            foreground: Rgba::hex(0x1f242c),
            cursor: Rgba::hex(0x2e86e0),
            selection: Rgba::hex(0xc9dcf5),
            normal: [
                Rgba::hex(0x3b4048),
                Rgba::hex(0xc0392f),
                Rgba::hex(0x2f7d32),
                Rgba::hex(0x8a6100),
                Rgba::hex(0x1f6ac0),
                Rgba::hex(0x8e44ad),
                Rgba::hex(0x00767d),
                Rgba::hex(0x545b66),
            ],
            bright: [
                Rgba::hex(0x6b7280),
                Rgba::hex(0xd44a3f),
                Rgba::hex(0x3d9142),
                Rgba::hex(0xa87400),
                Rgba::hex(0x2e86e0),
                Rgba::hex(0xa055c4),
                Rgba::hex(0x0a8f97),
                Rgba::hex(0x1f242c),
            ],
        }
    }
}
