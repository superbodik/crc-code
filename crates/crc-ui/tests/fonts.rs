use crc_theme::{Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::gpu::TextRun;
use crc_ui::{Frame, Offscreen, Quad};

const WIDTH: u32 = 420;
const HEIGHT: u32 = 120;

fn near(a: Rgba, b: Rgba) -> bool {
    a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
}

fn ink(run: TextRun) -> usize {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let theme = Theme::dark();
    let page = Rect::from_size(WIDTH as f32, HEIGHT as f32);

    let mut frame = Frame::new(theme.chrome.surface);
    frame.quad(Quad::filled(page, theme.chrome.surface));
    frame.text(run);

    let pixels = canvas.render_frame(&frame);
    canvas.count_pixels(&pixels, page, |c| near(c, theme.chrome.accent_solid))
}

fn run(text: &str) -> TextRun {
    TextRun::new(
        text,
        Rect::new(10.0, 10.0, 400.0, 100.0),
        40.0,
        Theme::dark().chrome.accent_solid,
    )
}

#[test]
fn the_shipped_faces_are_the_ones_that_get_used() {
    let canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let (sans, mono) = canvas.fonts();

    assert_eq!(sans, "IBM Plex Sans", "the vendored sans must win");
    assert_eq!(mono, "IBM Plex Mono", "the vendored mono must win");
}

#[test]
fn latin_letters_paint() {
    assert!(ink(run("Hello")) > 200, "latin came out blank");
}

#[test]
fn cyrillic_letters_paint() {
    assert!(ink(run("Привет")) > 200, "cyrillic came out blank");
}

#[test]
fn cyrillic_paints_in_the_mono_face_too() {
    assert!(
        ink(run("Привет").mono()) > 200,
        "the mono face has no cyrillic"
    );
}

#[test]
fn punctuation_and_dashes_survive_the_subset() {
    for text in ["\u{2014} \u{2013} \u{00ab}\u{00bb}", "\u{2192} \u{00d7} \u{2026}"] {
        assert!(ink(run(text)) > 40, "{text} came out blank");
    }
}

#[test]
fn a_heavier_weight_puts_down_more_ink() {
    let light = ink(run("Вес").weight(crc_theme::Weight::Regular));
    let heavy = ink(run("Вес").weight(crc_theme::Weight::Semibold));

    assert!(heavy > light, "semibold is not heavier: {light} vs {heavy}");
}
