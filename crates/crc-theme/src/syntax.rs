use crate::color::Rgba;
use crate::palette;

/// The highlight roles a tree-sitter query can resolve to.
///
/// Kept deliberately small. A theme with fifty roles looks like a rainbow and
/// nobody can hold it in their head; these are the distinctions that actually
/// help read code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Highlight {
    /// Plain code with no more specific role.
    Text,
    /// `import`, `const`, `fn`, `return`.
    Keyword,
    /// Function and type names.
    Function,
    /// Parameters, object keys, JSX attributes.
    Parameter,
    /// String and character literals.
    String,
    /// Numbers, booleans, `null`.
    Number,
    Comment,
    /// Brackets, operators, separators.
    Punctuation,
    /// Line numbers in the gutter.
    LineNumber,
    /// The line number of the line the cursor is on.
    LineNumberActive,
}

/// Colours for code.
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
    /// Behind the line the cursor is on.
    pub current_line: Rgba,
    /// Behind selected text.
    pub selection: Rgba,
    pub caret: Rgba,
    /// A collaborator's caret, before their colour is mixed in.
    pub peer_caret: Rgba,
}

impl SyntaxTheme {
    /// The design file's code colours, darkened where they did not clear 4.5:1
    /// against the buffer — keywords, parameters and strings all landed between
    /// 3.0 and 4.4, and those are the tokens a reader hits most. The hues are
    /// the design's; only the lightness moved. Chrome colours are untouched.
    pub const fn light() -> Self {
        Self {
            text: palette::NEUTRAL_850,
            keyword: palette::VIOLET_600,
            function: palette::BLUE_500,
            parameter: palette::AMBER_600,
            string: palette::GREEN_600,
            number: palette::GREEN_600,
            // Comments read as quiet but still have to be read, so they sit
            // above the line numbers rather than beside them.
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
