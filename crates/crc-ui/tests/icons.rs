use crc_theme::{Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::gpu::TextRun;
use crc_ui::icon;
use crc_ui::{Frame, Offscreen, Quad};

const WIDTH: u32 = 200;
const HEIGHT: u32 = 120;

fn near(a: Rgba, b: Rgba) -> bool {
    a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
}

fn draw(glyph: char, size: f32) -> (Offscreen, Vec<u8>, Rect) {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let theme = Theme::dark();
    let box_of = Rect::new(20.0, 20.0, 80.0, 80.0);

    let mut frame = Frame::new(theme.chrome.surface);
    frame.quad(Quad::filled(
        Rect::from_size(WIDTH as f32, HEIGHT as f32),
        theme.chrome.surface,
    ));
    frame.text(TextRun::icon(glyph, box_of, size, theme.chrome.accent_solid));

    let pixels = canvas.render_frame(&frame);
    (canvas, pixels, box_of)
}

#[test]
fn an_icon_actually_paints() {
    let theme = Theme::dark();
    let (canvas, pixels, box_of) = draw(icon::FOLDER, 48.0);

    let lit = canvas.count_pixels(&pixels, box_of, |c| near(c, theme.chrome.accent_solid));
    assert!(
        lit > 100,
        "the icon font did not load or the glyph is empty: {lit} pixels"
    );
}

#[test]
fn every_icon_in_the_set_has_a_shape() {
    let theme = Theme::dark();
    let all = [
        ("folder", icon::FOLDER),
        ("folder open", icon::FOLDER_OPEN),
        ("file", icon::FILE),
        ("code", icon::FILE_CODE),
        ("explorer", icon::EXPLORER),
        ("search", icon::SEARCH),
        ("git", icon::GIT),
        ("gear", icon::GEAR),
        ("terminal", icon::TERMINAL),
        ("problems", icon::PROBLEMS),
        ("close", icon::CLOSE),
        ("chevron", icon::CHEVRON_RIGHT),
        ("dot", icon::DOT),
        ("robot", icon::ROBOT),
        ("keyboard", icon::KEYBOARD),
        ("palette", icon::PALETTE),
    ];

    for (name, glyph) in all {
        let (canvas, pixels, box_of) = draw(glyph, 48.0);
        let lit = canvas.count_pixels(&pixels, box_of, |c| near(c, theme.chrome.accent_solid));
        assert!(lit > 50, "{name} came out blank: {lit} pixels");
    }
}

#[test]
fn a_bigger_icon_covers_more_ground() {
    let theme = Theme::dark();
    let lit = |size: f32| {
        let (canvas, pixels, box_of) = draw(icon::FOLDER, size);
        canvas.count_pixels(&pixels, box_of, |c| near(c, theme.chrome.accent_solid))
    };

    assert!(lit(48.0) > lit(16.0), "the icon does not scale");
}

mod naming {
    use super::*;

    #[test]
    fn a_file_gets_the_icon_of_its_language() {
        assert_eq!(icon::for_name("main.rs"), icon::FILE_CODE);
        assert_eq!(icon::for_name("store.ts"), icon::FILETYPE_TS);
        assert_eq!(icon::for_name("app.py"), icon::FILETYPE_PY);
        assert_eq!(icon::for_name("index.html"), icon::FILETYPE_HTML);
        assert_eq!(icon::for_name("style.css"), icon::FILETYPE_CSS);
        assert_eq!(icon::for_name("palette.json"), icon::FILETYPE_JSON);
        assert_eq!(icon::for_name("Cargo.toml"), icon::FILE_TEXT);
    }

    #[test]
    fn the_extension_is_read_without_regard_to_case() {
        assert_eq!(icon::for_name("README.MD"), icon::FILETYPE_MD);
        assert_eq!(icon::for_name("Photo.PNG"), icon::FILE_IMAGE);
    }

    #[test]
    fn a_name_with_no_extension_gets_the_plain_sheet() {
        assert_eq!(icon::for_name("Makefile"), icon::FILE);
        assert_eq!(icon::for_name("LICENSE"), icon::FILE);
    }

    #[test]
    fn a_dotted_name_reads_its_last_piece() {
        assert_eq!(icon::for_name("crc-icons.subset.ttf"), icon::FILE);
        assert_eq!(icon::for_name("vite.config.ts"), icon::FILETYPE_TS);
    }
}
