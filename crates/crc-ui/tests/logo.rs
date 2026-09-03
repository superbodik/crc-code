use crc_theme::{Brand, Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::logo::{self, CUT_BELOW};
use crc_ui::{Frame, Offscreen};

fn near(a: Rgba, b: Rgba) -> bool {
    a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
}

#[test]
fn the_mark_follows_the_proportions_from_the_brand_sheet() {
    let side = 128.0;
    let mark = logo::mark(side, 0.0, 0.0);

    let percent = |value: f32| value / side;
    assert!((percent(mark.radius) - 0.20).abs() < 0.001, "radius");
    assert!((percent(mark.cut) - 0.26).abs() < 0.001, "corner cut");
    assert!(
        (percent(mark.caret.width) - 0.065).abs() < 0.001,
        "caret width"
    );
    assert!(
        (percent(mark.caret.height) - 0.44).abs() < 0.001,
        "caret height"
    );
    assert!(
        (percent(logo::clear_space(side)) - 0.12).abs() < 0.001,
        "clear space"
    );
}

#[test]
fn the_proportions_hold_at_every_size_in_the_ramp() {
    for side in [128.0, 64.0, 32.0] {
        let mark = logo::mark(side, 0.0, 0.0);
        assert!(
            (mark.radius / side - 0.20).abs() < 0.001,
            "radius drifted at {side}"
        );
        assert!(
            (mark.caret.height / side - 0.44).abs() < 0.001,
            "caret drifted at {side}"
        );
    }
}

#[test]
fn the_corner_cut_goes_away_at_small_sizes() {
    assert!(
        logo::mark(CUT_BELOW, 0.0, 0.0).cut > 0.0,
        "the cut survives at the threshold"
    );
    assert_eq!(
        logo::mark(16.0, 0.0, 0.0).cut,
        0.0,
        "at 16px the sheet drops the cut"
    );
    assert!(
        logo::mark(16.0, 0.0, 0.0).radius > 0.0,
        "the radius stays even when the cut goes"
    );
}

#[test]
fn the_mark_lands_where_it_was_placed() {
    let mark = logo::mark(40.0, 100.0, 50.0);

    assert_eq!(mark.block, Rect::new(100.0, 50.0, 40.0, 40.0));
    assert!(mark.caret.x > mark.block.x + mark.block.width * 0.5);
    assert!(mark.caret.right() < mark.block.right());
    assert!(mark.caret.y > mark.block.y);
    assert!(mark.caret.bottom() < mark.block.bottom());
}

#[test]
fn the_caret_sits_right_of_the_monogram() {
    let mark = logo::mark(64.0, 0.0, 0.0);
    assert!(
        mark.caret.x > mark.glyph_center,
        "the caret reads as the cursor after the letter"
    );
}

#[test]
fn a_zero_sized_mark_draws_nothing() {
    let mut frame = Frame::new(Rgba::hex(0xffffff));
    logo::draw(&mut frame, logo::mark(0.0, 10.0, 10.0), Brand::colour());

    assert!(frame.quads().is_empty());
    assert!(frame.runs().is_empty());
}

#[test]
fn the_mark_paints_its_block_the_caret_and_the_letter() {
    let mut canvas = Offscreen::new(160, 160).expect("a GPU");
    let brand = Brand::colour();

    let mut frame = Frame::new(Rgba::hex(0xffffff));
    logo::draw(&mut frame, logo::mark(120.0, 20.0, 20.0), brand);
    let pixels = canvas.render_frame(&frame);

    let block = Rect::new(20.0, 20.0, 120.0, 120.0);
    assert!(
        canvas.count_pixels(&pixels, block, |c| near(c, brand.mark)) > 5000,
        "the block did not paint"
    );
    assert!(
        canvas.count_pixels(&pixels, block, |c| near(c, brand.glyph)) > 200,
        "neither the letter nor the caret painted"
    );
    assert!(
        near(canvas.pixel(&pixels, 5, 5), Rgba::hex(0xffffff)),
        "the mark spilled outside its block"
    );
}

#[test]
fn the_dark_variant_inverts_the_block_and_keeps_the_caret_brand_coloured() {
    let colour = Brand::colour();
    let dark = Brand::on_dark();

    assert_eq!(dark.mark, colour.glyph, "the ink becomes the block");
    assert_eq!(
        dark.caret, colour.mark,
        "the caret carries the brand colour"
    );
    assert_ne!(dark.glyph, dark.mark, "the letter still reads on the block");
}

#[test]
fn every_variant_keeps_the_letter_readable_on_its_block() {
    for (name, brand) in [
        ("colour", Brand::colour()),
        ("dark", Brand::on_dark()),
        ("monochrome", Brand::monochrome()),
    ] {
        let ratio = brand.glyph.contrast_ratio(brand.mark);
        assert!(
            ratio >= 4.5,
            "{name}: the monogram is {ratio:.2}:1 against its block"
        );
    }
}

#[test]
fn the_caret_reads_against_the_block_in_every_variant() {
    for (name, brand) in [
        ("colour", Brand::colour()),
        ("dark", Brand::on_dark()),
        ("monochrome", Brand::monochrome()),
    ] {
        let ratio = brand.caret.contrast_ratio(brand.mark);
        assert!(ratio >= 3.0, "{name}: the caret is {ratio:.2}:1");
    }
}

#[test]
fn the_family_shares_lightness_and_chroma_with_the_team_mark() {
    let crc = crc_theme::brand::MARK.relative_luminance();
    let team = crc_theme::brand::TEAM.relative_luminance();

    assert!(
        (crc - team).abs() < 0.09,
        "the two marks should read at the same weight: {crc:.3} vs {team:.3}"
    );
}

#[test]
fn the_title_bar_carries_the_mark() {
    use crc_ui::view::{self, CodeMetrics, EditorView};
    use crc_ui::{Shell, ShellState};

    let mut canvas = Offscreen::new(640, 400).expect("a GPU");
    let theme = Theme::light();
    let layout = Shell::compute(
        Rect::from_size(640.0, 400.0),
        &theme,
        &ShellState::default(),
    );

    let view = EditorView {
        project: "crc-code".to_string(),
        focused: true,
        ..EditorView::default()
    };
    let pixels = canvas.render_frame(&view::draw(&layout, &theme, &view, CodeMetrics::default()));

    assert!(
        canvas.count_pixels(&pixels, layout.titlebar, |c| near(c, Brand::colour().mark)) > 60,
        "the logo is missing from the title bar"
    );
}
