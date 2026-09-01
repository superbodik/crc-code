#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontFamily {
    pub primary: &'static str,
    pub fallbacks: &'static [&'static str],
}

pub const SANS: FontFamily = FontFamily {
    primary: "IBM Plex Sans",
    fallbacks: &["Segoe UI", "Helvetica", "sans-serif"],
};

pub const MONO: FontFamily = FontFamily {
    primary: "IBM Plex Mono",
    fallbacks: &["Cascadia Mono", "Consolas", "monospace"],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    Regular = 400,
    Medium = 500,
    Semibold = 600,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeScale {
    pub micro: f32,
    pub tiny: f32,
    pub small: f32,
    pub body: f32,
    pub code: f32,
    pub large: f32,
    pub title: f32,
    pub heading: f32,
    pub subdisplay: f32,
    pub display: f32,
}

impl TypeScale {
    pub const fn default_scale() -> Self {
        Self {
            micro: 9.0,
            tiny: 10.0,
            small: 11.0,
            body: 12.0,
            code: 13.0,
            large: 14.0,
            title: 15.0,
            heading: 20.0,
            subdisplay: 24.0,
            display: 32.0,
        }
    }

    pub fn scaled(self, factor: f32) -> Self {
        Self {
            micro: self.micro * factor,
            tiny: self.tiny * factor,
            small: self.small * factor,
            body: self.body * factor,
            code: self.code * factor,
            large: self.large * factor,
            title: self.title * factor,
            heading: self.heading * factor,
            subdisplay: self.subdisplay * factor,
            display: self.display * factor,
        }
    }
}

impl Default for TypeScale {
    fn default() -> Self {
        Self::default_scale()
    }
}

pub const LINE_HEIGHT_UI: f32 = 1.4;
pub const LINE_HEIGHT_CODE: f32 = 1.55;

const _: () = assert!(LINE_HEIGHT_CODE > LINE_HEIGHT_UI);
