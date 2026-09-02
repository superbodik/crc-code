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
            text: palette::light::NEUTRAL_850,
            keyword: palette::light::VIOLET_600,
            function: palette::light::BLUE_500,
            parameter: palette::light::AMBER_600,
            string: palette::light::GREEN_600,
            number: palette::light::GREEN_600,
            comment: palette::light::NEUTRAL_700,
            punctuation: palette::light::NEUTRAL_800,
            line_number: palette::light::NEUTRAL_600,
            line_number_active: palette::light::NEUTRAL_850,
            current_line: palette::light::NEUTRAL_100,
            selection: palette::light::ACCENT_100,
            caret: palette::light::ACCENT_500,
            peer_caret: palette::light::ACCENT_400,
        }
    }

    pub const fn dark() -> Self {
        Self {
            text: palette::dark::NEUTRAL_850,
            keyword: palette::dark::VIOLET_500,
            function: palette::dark::BLUE_500,
            parameter: palette::dark::AMBER_500,
            string: palette::dark::GREEN_500,
            number: palette::dark::GREEN_500,
            comment: palette::dark::NEUTRAL_600,
            punctuation: palette::dark::NEUTRAL_800,
            line_number: palette::dark::NEUTRAL_550,
            line_number_active: palette::dark::NEUTRAL_850,
            current_line: palette::dark::NEUTRAL_150,
            selection: palette::dark::ACCENT_100,
            caret: palette::dark::ACCENT_500,
            peer_caret: palette::dark::ACCENT_400,
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
