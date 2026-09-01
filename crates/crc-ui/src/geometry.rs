#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn from_size(width: f32, height: f32) -> Self {
        Self::new(0.0, 0.0, width, height)
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    pub fn inset(&self, amount: f32) -> Rect {
        self.inset_by(amount, amount)
    }

    pub fn inset_by(&self, horizontal: f32, vertical: f32) -> Rect {
        Rect::new(
            self.x + horizontal,
            self.y + vertical,
            (self.width - horizontal * 2.0).max(0.0),
            (self.height - vertical * 2.0).max(0.0),
        )
    }

    pub fn split_top(&self, height: f32) -> (Rect, Rect) {
        let height = height.clamp(0.0, self.height);
        (
            Rect::new(self.x, self.y, self.width, height),
            Rect::new(self.x, self.y + height, self.width, self.height - height),
        )
    }

    pub fn split_bottom(&self, height: f32) -> (Rect, Rect) {
        let height = height.clamp(0.0, self.height);
        (
            Rect::new(self.x, self.bottom() - height, self.width, height),
            Rect::new(self.x, self.y, self.width, self.height - height),
        )
    }

    pub fn split_left(&self, width: f32) -> (Rect, Rect) {
        let width = width.clamp(0.0, self.width);
        (
            Rect::new(self.x, self.y, width, self.height),
            Rect::new(self.x + width, self.y, self.width - width, self.height),
        )
    }

    pub fn split_right(&self, width: f32) -> (Rect, Rect) {
        let width = width.clamp(0.0, self.width);
        (
            Rect::new(self.right() - width, self.y, width, self.height),
            Rect::new(self.x, self.y, self.width - width, self.height),
        )
    }

    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right <= x || bottom <= y {
            return None;
        }
        Some(Rect::new(x, y, right - x, bottom - y))
    }
}
