pub const FAMILY: &str = "CRC Icons";

pub const FOLDER: char = '\u{f3d9}';
pub const FOLDER_OPEN: char = '\u{f3d8}';
pub const FILE: char = '\u{f392}';
pub const FILE_CODE: char = '\u{f362}';
pub const FILE_TEXT: char = '\u{f38b}';
pub const FILE_IMAGE: char = '\u{f36d}';
pub const FILE_BINARY: char = '\u{f35c}';
pub const FILE_LOCK: char = '\u{f36f}';
pub const FILETYPE_JS: char = '\u{f74c}';
pub const FILETYPE_TS: char = '\u{f764}';
pub const FILETYPE_JSON: char = '\u{f791}';
pub const FILETYPE_HTML: char = '\u{f749}';
pub const FILETYPE_CSS: char = '\u{f742}';
pub const FILETYPE_PY: char = '\u{f75c}';
pub const FILETYPE_JAVA: char = '\u{f74a}';
pub const FILETYPE_MD: char = '\u{f750}';
pub const FILETYPE_YML: char = '\u{f76c}';
pub const FILETYPE_XML: char = '\u{f76b}';
pub const FILETYPE_SVG: char = '\u{f762}';
pub const FILETYPE_TXT: char = '\u{f766}';
pub const FILETYPE_SH: char = '\u{f761}';
pub const EXPLORER: char = '\u{f3c2}';
pub const SEARCH: char = '\u{f52a}';
pub const GIT: char = '\u{f69d}';
pub const GEAR: char = '\u{f3e5}';
pub const ROBOT: char = '\u{f6b1}';
pub const TERMINAL: char = '\u{f5c3}';
pub const PROBLEMS: char = '\u{f33b}';
pub const OUTPUT: char = '\u{f478}';
pub const TESTS: char = '\u{f271}';
pub const NEW_FILE: char = '\u{f37d}';
pub const NEW_FOLDER: char = '\u{f3d3}';
pub const RENAME: char = '\u{f4cb}';
pub const DELETE: char = '\u{f78b}';
pub const COPY_PATH: char = '\u{f290}';
pub const REVEAL: char = '\u{f1c5}';
pub const REFRESH: char = '\u{f116}';
pub const CHEVRON_RIGHT: char = '\u{f285}';
pub const CHEVRON_UP: char = '\u{f286}';
pub const MATCH_CASE: char = '\u{f5f7}';
pub const WHOLE_WORD: char = '\u{f1c9}';
pub const REGEX: char = '\u{f151}';
pub const REPLACE: char = '\u{f12b}';
pub const CHEVRON_DOWN: char = '\u{f282}';
pub const CLOSE: char = '\u{f659}';
pub const MINIMIZE: char = '\u{f63b}';
pub const MAXIMIZE: char = '\u{f64d}';
pub const PLUS: char = '\u{f4fe}';
pub const RESET: char = '\u{f117}';
pub const DOT: char = '\u{f287}';
pub const CHECK: char = '\u{f633}';
pub const WARNING: char = '\u{f333}';
pub const INFO: char = '\u{f431}';
pub const BRANCH: char = '\u{f2ec}';
pub const SAVE: char = '\u{f7d8}';
pub const OPEN_FOLDER: char = '\u{f3d8}';
pub const OPEN_FILE: char = '\u{f358}';
pub const CLOCK: char = '\u{f292}';
pub const SPLIT: char = '\u{f460}';
pub const ZEN: char = '\u{f3df}';
pub const THEME: char = '\u{f288}';
pub const KEYBOARD: char = '\u{f451}';
pub const PALETTE: char = '\u{f2cf}';

pub fn for_extension(extension: &str) -> char {
    match extension {
        "rs" => FILE_CODE,
        "js" | "mjs" | "cjs" => FILETYPE_JS,
        "ts" | "tsx" | "jsx" => FILETYPE_TS,
        "json" => FILETYPE_JSON,
        "html" | "htm" => FILETYPE_HTML,
        "css" | "scss" | "sass" => FILETYPE_CSS,
        "py" | "pyi" => FILETYPE_PY,
        "java" => FILETYPE_JAVA,
        "md" | "markdown" => FILETYPE_MD,
        "yml" | "yaml" => FILETYPE_YML,
        "xml" => FILETYPE_XML,
        "svg" => FILETYPE_SVG,
        "txt" | "log" => FILETYPE_TXT,
        "sh" | "bash" | "bat" | "cmd" | "ps1" => FILETYPE_SH,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" | "bmp" => FILE_IMAGE,
        "exe" | "dll" | "so" | "dylib" | "bin" | "wasm" => FILE_BINARY,
        "toml" | "ini" | "cfg" | "conf" | "lock" => FILE_TEXT,
        "c" | "h" | "cpp" | "cc" | "hpp" | "cs" | "go" | "rb" | "php" | "swift" | "kt" => FILE_CODE,
        "pem" | "key" | "crt" | "env" => FILE_LOCK,
        _ => FILE,
    }
}

pub fn for_name(name: &str) -> char {
    let lowered = name.to_lowercase();
    match lowered.rsplit_once('.') {
        Some((_, extension)) => for_extension(extension),
        None => FILE,
    }
}
