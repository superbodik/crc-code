use crc_syntax::{Language, SyntaxTree};
use crc_theme::{Rgba, Theme, Weight};
use crc_ui::geometry::Rect;
use crc_ui::gpu::text::segments;
use crc_ui::{Frame, Offscreen, Quad, Span, TextRun};

const SIZE: u32 = 256;

fn canvas() -> Offscreen {
    Offscreen::new(SIZE, SIZE).expect("a GPU adapter is available")
}

fn area() -> Rect {
    Rect::new(10.0, 10.0, 236.0, 120.0)
}

fn is_dark(colour: Rgba) -> bool {
    colour.r < 100 && colour.g < 100 && colour.b < 100
}

fn has_ink(colour: Rgba) -> bool {
    colour.r < 200 || colour.g < 200 || colour.b < 200
}

fn is_reddish(colour: Rgba) -> bool {
    colour.r > 120 && colour.g < 90 && colour.b < 90
}

fn is_blueish(colour: Rgba) -> bool {
    colour.b > 120 && colour.r < 90 && colour.g < 90
}

#[test]
fn a_font_is_resolved_for_both_families() {
    let canvas = canvas();
    let (sans, mono) = canvas.fonts();

    println!("sans: {sans}, mono: {mono}");
    assert!(!sans.is_empty());
    assert!(!mono.is_empty());
}

#[test]
fn text_actually_reaches_the_pixels() {
    let mut canvas = canvas();
    let white = Rgba::hex(0xffffff);

    let blank = canvas.render_frame(&Frame::new(white));
    let written = canvas.render_frame(&Frame::new(white).with_text([TextRun::new(
        "Hello",
        area(),
        48.0,
        Rgba::hex(0x000000),
    )]));

    assert_eq!(canvas.count_pixels(&blank, area(), is_dark), 0);
    assert!(
        canvas.count_pixels(&written, area(), is_dark) > 50,
        "the glyphs left no dark pixels behind"
    );
}

#[test]
fn text_stays_inside_the_rect_it_was_given() {
    let mut canvas = canvas();
    let strip = Rect::new(0.0, 0.0, 256.0, 60.0);

    let frame = canvas.render_frame(&Frame::new(Rgba::hex(0xffffff)).with_text([TextRun::new(
        "clipped to the strip",
        strip,
        40.0,
        Rgba::hex(0x000000),
    )]));

    let below = Rect::new(0.0, 70.0, 256.0, 186.0);
    assert_eq!(
        canvas.count_pixels(&frame, below, is_dark),
        0,
        "text escaped below its bounds"
    );
}

#[test]
fn spans_colour_their_own_stretch_of_text() {
    let mut canvas = canvas();

    let frame = canvas.render_frame(
        &Frame::new(Rgba::hex(0xffffff)).with_text([TextRun::new(
            "AAABBB",
            area(),
            48.0,
            Rgba::hex(0x000000),
        )
        .mono()
        .spans(vec![
            Span::new(0..3, Rgba::hex(0xcc0000)),
            Span::new(3..6, Rgba::hex(0x0000cc)),
        ])]),
    );

    assert!(
        canvas.count_pixels(&frame, area(), is_reddish) > 20,
        "the first span did not come out red"
    );
    assert!(
        canvas.count_pixels(&frame, area(), is_blueish) > 20,
        "the second span did not come out blue"
    );
}

#[test]
fn text_draws_over_the_quads_beneath_it() {
    let mut canvas = canvas();
    let panel = Rgba::hex(0xffffff);

    let frame = canvas.render_frame(
        &Frame::new(Rgba::hex(0x000000))
            .with_quads([Quad::filled(Rect::new(0.0, 0.0, 256.0, 256.0), panel)])
            .with_text([TextRun::new("over", area(), 48.0, Rgba::hex(0x000000))]),
    );

    assert!(canvas.count_pixels(&frame, area(), is_dark) > 50);
}

#[test]
fn measuring_grows_with_the_text_and_the_size() {
    let mut canvas = canvas();
    let rect = Rect::new(0.0, 0.0, 1000.0, 100.0);

    let short = canvas.measure(&TextRun::new("ab", rect, 16.0, Rgba::hex(0x000000)));
    let long = canvas.measure(&TextRun::new("abcdefghij", rect, 16.0, Rgba::hex(0x000000)));
    let bigger = canvas.measure(&TextRun::new("ab", rect, 32.0, Rgba::hex(0x000000)));

    assert!(short.0 > 0.0, "an empty measurement is not a measurement");
    assert!(long.0 > short.0);
    assert!(bigger.0 > short.0);
    assert!(bigger.1 > short.1);
}

#[test]
fn a_monospace_run_advances_evenly() {
    let mut canvas = canvas();
    let rect = Rect::new(0.0, 0.0, 1000.0, 100.0);

    let one = canvas
        .measure(&TextRun::new("i", rect, 20.0, Rgba::hex(0x000000)).mono())
        .0;
    let other = canvas
        .measure(&TextRun::new("m", rect, 20.0, Rgba::hex(0x000000)).mono())
        .0;

    assert!(
        (one - other).abs() < 0.5,
        "i and m should take the same width in a mono font, got {one} and {other}"
    );
}

#[test]
fn a_highlighted_line_of_code_renders_in_its_theme_colours() {
    let mut canvas = canvas();
    let theme = Theme::light();
    let source = "const path = 'src'";

    let mut tree = SyntaxTree::new(Language::TypeScript).unwrap();
    tree.parse(source).unwrap();

    let spans: Vec<Span> = tree
        .highlights(source)
        .into_iter()
        .map(|span| Span::new(span.range, theme.syntax.color(span.highlight)))
        .collect();

    assert!(!spans.is_empty(), "the line produced no highlights");

    let frame = canvas.render_frame(
        &Frame::new(theme.chrome.surface).with_text([TextRun::new(
            source,
            area(),
            20.0,
            theme.syntax.text,
        )
        .mono()
        .spans(spans)]),
    );

    let keyword = theme.syntax.keyword;
    let near_keyword = |c: Rgba| {
        c.r.abs_diff(keyword.r) < 40 && c.g.abs_diff(keyword.g) < 40 && c.b.abs_diff(keyword.b) < 40
    };
    assert!(
        canvas.count_pixels(&frame, area(), near_keyword) > 5,
        "const did not come out in the keyword colour"
    );
}

#[test]
fn many_runs_share_one_atlas() {
    let mut canvas = canvas();

    let runs: Vec<TextRun> = (0..12)
        .map(|i| {
            TextRun::new(
                format!("line {i}"),
                Rect::new(4.0, i as f32 * 20.0, 240.0, 20.0),
                14.0,
                Rgba::hex(0x000000),
            )
            .mono()
        })
        .collect();

    let frame = canvas.render_frame(&Frame::new(Rgba::hex(0xffffff)).with_text(runs));

    for row in 0..12 {
        let line = Rect::new(0.0, row as f32 * 20.0, 256.0, 20.0);
        assert!(
            canvas.count_pixels(&frame, line, has_ink) > 10,
            "row {row} came out blank"
        );
    }
}

#[test]
fn an_empty_run_draws_nothing_and_does_not_panic() {
    let mut canvas = canvas();
    let white = Rgba::hex(0xffffff);

    let frame = canvas.render_frame(&Frame::new(white).with_text([
        TextRun::new("", area(), 20.0, Rgba::hex(0x000000)),
        TextRun::new("x", Rect::ZERO, 20.0, Rgba::hex(0x000000)),
    ]));

    assert_eq!(canvas.count_pixels(&frame, area(), is_dark), 0);
}

#[test]
fn weight_changes_how_much_ink_lands_on_the_page() {
    let mut canvas = canvas();
    let white = Rgba::hex(0xffffff);
    let black = Rgba::hex(0x000000);

    let regular = canvas.render_frame(&Frame::new(white).with_text([TextRun::new(
        "Weight",
        area(),
        40.0,
        black,
    )]));
    let semibold = canvas.render_frame(
        &Frame::new(white)
            .with_text([TextRun::new("Weight", area(), 40.0, black).weight(Weight::Semibold)]),
    );

    let thin = canvas.count_pixels(&regular, area(), is_dark);
    let thick = canvas.count_pixels(&semibold, area(), is_dark);
    assert!(thin > 0 && thick > 0);
    assert!(
        thick >= thin,
        "semibold laid down less ink than regular: {thick} vs {thin}"
    );
}

mod splitting {
    use super::*;

    fn coloured(text: &str, spans: Vec<Span>) -> Vec<(String, Rgba)> {
        segments(text, &spans, Rgba::hex(0x000000))
            .into_iter()
            .map(|(piece, colour)| (piece.to_string(), colour))
            .collect()
    }

    #[test]
    fn text_with_no_spans_is_one_piece() {
        let out = coloured("hello", vec![]);
        assert_eq!(out, vec![("hello".to_string(), Rgba::hex(0x000000))]);
    }

    #[test]
    fn gaps_between_spans_take_the_default_colour() {
        let red = Rgba::hex(0xff0000);
        let out = coloured("abcdef", vec![Span::new(2..4, red)]);

        assert_eq!(
            out,
            vec![
                ("ab".to_string(), Rgba::hex(0x000000)),
                ("cd".to_string(), red),
                ("ef".to_string(), Rgba::hex(0x000000)),
            ]
        );
    }

    #[test]
    fn the_pieces_always_rebuild_the_original_text() {
        let red = Rgba::hex(0xff0000);
        for spans in [
            vec![],
            vec![Span::new(0..6, red)],
            vec![Span::new(0..2, red), Span::new(4..6, red)],
            vec![Span::new(3..3, red)],
            vec![Span::new(2..99, red)],
        ] {
            let joined: String = coloured("abcdef", spans.clone())
                .into_iter()
                .map(|(piece, _)| piece)
                .collect();
            assert_eq!(joined, "abcdef", "lost text with spans {spans:?}");
        }
    }

    #[test]
    fn a_span_landing_mid_character_does_not_split_it() {
        let red = Rgba::hex(0xff0000);
        let out = coloured("привет", vec![Span::new(0..3, red)]);

        let joined: String = out.iter().map(|(piece, _)| piece.as_str()).collect();
        assert_eq!(joined, "привет");
        assert_eq!(out[0].0, "п", "a two-byte character stayed whole");
    }

    #[test]
    fn a_span_running_past_the_end_is_clamped() {
        let red = Rgba::hex(0xff0000);
        let out = coloured("abc", vec![Span::new(1..500, red)]);

        assert_eq!(
            out,
            vec![
                ("a".to_string(), Rgba::hex(0x000000)),
                ("bc".to_string(), red),
            ]
        );
    }

    #[test]
    fn overlapping_spans_do_not_duplicate_text() {
        let red = Rgba::hex(0xff0000);
        let blue = Rgba::hex(0x0000ff);
        let out = coloured("abcdef", vec![Span::new(0..4, red), Span::new(2..6, blue)]);

        let joined: String = out.iter().map(|(piece, _)| piece.as_str()).collect();
        assert_eq!(joined, "abcdef");
    }
}
