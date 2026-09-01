use crate::color::Rgba;
use crate::palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Highlight {
    Text,
    Keyword,
    Function,
    Parameter,
    String,
    Number,
    Comment,
    Punctuation,
    LineNumber,
    LineNumberActive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SyntaxTheme {
    pub text: Rgba,
    pub keyword: Rgba,
    pub function: Rgba,
    pub parameter: Rgba,
    pub string: Rgba,
    pub number: Rgba,
    pub comment: Rgba,
    pub punctuation: Rgba,
    pub line_number: Rgba,
    pub line_number_active: Rgba,
    pub current_line: Rgba,
    pub selection: Rgba,
    pub caret: Rgba,
    pub peer_caret: Rgba,
}

impl SyntaxTheme {
    pub const fn light() -> Self {
        Self {
            text: palette::NEUTRAL_850,
            keyword: palette::VIOLET_600,
            function: palette::BLUE_500,
            parameter: palette::AMBER_600,
            string: palette::GREEN_600,
            number: palette::GREEN_600,
            comment: palette::NEUTRAL_700,
            punctuation: palette::NEUTRAL_800,
            line_number: palette::NEUTRAL_600,
            line_number_active: palette::NEUTRAL_850,
            current_line: palette::NEUTRAL_100,
            selection: palette::ACCENT_100,
            caret: palette::ACCENT_500,
            peer_caret: palette::ACCENT_400,
        }
    }

    pub const fn color(&self, highlight: Highlight) -> Rgba {
        match highlight {
            Highlight::Text => self.text,
            Highlight::Keyword => self.keyword,
            Highlight::Function => self.function,
            Highlight::Parameter => self.parameter,
            Highlight::String => self.string,
            Highlight::Number => self.number,
            Highlight::Comment => self.comment,
            Highlight::Punctuation => self.punctuation,
            Highlight::LineNumber => self.line_number,
            Highlight::LineNumberActive => self.line_number_active,
        }
    }
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self::light()
    }
}
