use crc_theme::{Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::{Offscreen, Quad};

const SIZE: u32 = 256;

fn canvas() -> Offscreen {
    Offscreen::new(SIZE, SIZE).expect("a GPU adapter is available")
}

#[track_caller]
fn assert_colour(actual: Rgba, expected: Rgba) {
    let close = |a: u8, b: u8| a.abs_diff(b) <= 2;
    assert!(
        close(actual.r, expected.r) && close(actual.g, expected.g) && close(actual.b, expected.b),
        "expected {expected:?}, got {actual:?}"
    );
}

#[test]
fn the_pipeline_builds_and_runs_on_this_machine() {
    let mut canvas = canvas();
    println!("rendering on {}", canvas.adapter());

    let frame = canvas.render(Rgba::hex(0x123456), &[]);

    assert_eq!(frame.len(), (SIZE * SIZE * 4) as usize);
    assert_colour(canvas.pixel(&frame, 128, 128), Rgba::hex(0x123456));
}

#[test]
fn a_quad_paints_its_own_area_and_nothing_else() {
    let mut canvas = canvas();
    let background = Rgba::hex(0xffffff);
    let fill = Rgba::hex(0x7c5cff);

    let frame = canvas.render(
        background,
        &[Quad::filled(Rect::new(64.0, 64.0, 128.0, 128.0), fill)],
    );

    assert_colour(canvas.pixel(&frame, 128, 128), fill);
    assert_colour(canvas.pixel(&frame, 70, 70), fill);
    assert_colour(canvas.pixel(&frame, 10, 10), background);
    assert_colour(canvas.pixel(&frame, 250, 250), background);
    assert_colour(canvas.pixel(&frame, 128, 10), background);
}

#[test]
fn a_quad_lands_where_the_layout_put_it() {
    let mut canvas = canvas();
    let fill = Rgba::hex(0x000000);

    let frame = canvas.render(
        Rgba::hex(0xffffff),
        &[Quad::filled(Rect::new(0.0, 0.0, 256.0, 40.0), fill)],
    );

    assert_colour(canvas.pixel(&frame, 128, 20), fill);
    assert_colour(canvas.pixel(&frame, 128, 45), Rgba::hex(0xffffff));
}

#[test]
fn a_border_draws_on_the_edge_with_the_fill_inside() {
    let mut canvas = canvas();
    let fill = Rgba::hex(0xffffff);
    let border = Rgba::hex(0x7c5cff);

    let frame = canvas.render(
        Rgba::hex(0x000000),
        &[Quad::filled(Rect::new(28.0, 28.0, 200.0, 200.0), fill).bordered(6.0, border)],
    );

    assert_colour(canvas.pixel(&frame, 31, 128), border);
    assert_colour(canvas.pixel(&frame, 128, 31), border);
    assert_colour(canvas.pixel(&frame, 128, 128), fill);
}

#[test]
fn a_rounded_corner_leaves_the_corner_empty() {
    let mut canvas = canvas();
    let background = Rgba::hex(0x000000);
    let fill = Rgba::hex(0xffffff);

    let frame = canvas.render(
        background,
        &[Quad::filled(Rect::new(28.0, 28.0, 200.0, 200.0), fill).rounded(40.0)],
    );

    assert_colour(canvas.pixel(&frame, 30, 30), background);
    assert_colour(canvas.pixel(&frame, 128, 30), fill);
    assert_colour(canvas.pixel(&frame, 30, 128), fill);
    assert_colour(canvas.pixel(&frame, 128, 128), fill);
}

#[test]
fn a_radius_larger_than_the_quad_does_not_fold_it_inside_out() {
    let mut canvas = canvas();
    let fill = Rgba::hex(0xffffff);

    let frame = canvas.render(
        Rgba::hex(0x000000),
        &[Quad::filled(Rect::new(108.0, 108.0, 40.0, 40.0), fill).rounded(500.0)],
    );

    assert_colour(canvas.pixel(&frame, 128, 128), fill);
    assert_colour(canvas.pixel(&frame, 109, 109), Rgba::hex(0x000000));
}

#[test]
fn later_quads_draw_over_earlier_ones() {
    let mut canvas = canvas();
    let under = Rgba::hex(0xff0000);
    let over = Rgba::hex(0x0000ff);

    let frame = canvas.render(
        Rgba::hex(0xffffff),
        &[
            Quad::filled(Rect::new(32.0, 32.0, 160.0, 160.0), under),
            Quad::filled(Rect::new(96.0, 96.0, 128.0, 128.0), over),
        ],
    );

    assert_colour(canvas.pixel(&frame, 128, 128), over);
    assert_colour(canvas.pixel(&frame, 40, 40), under);
}

#[test]
fn a_translucent_quad_lets_what_is_under_it_through() {
    let mut canvas = canvas();

    let frame = canvas.render(
        Rgba::hex(0x000000),
        &[Quad::filled(
            Rect::new(32.0, 32.0, 192.0, 192.0),
            Rgba::hex(0xffffff).with_alpha(128),
        )],
    );

    let blended = canvas.pixel(&frame, 128, 128);
    assert!(
        blended.r > 20 && blended.r < 235,
        "half-transparent white over black should land between the two, got {blended:?}"
    );
}

#[test]
fn an_empty_quad_draws_nothing() {
    let mut canvas = canvas();
    let background = Rgba::hex(0x336699);

    let frame = canvas.render(
        background,
        &[
            Quad::filled(Rect::new(10.0, 10.0, 0.0, 100.0), Rgba::hex(0xff0000)),
            Quad::filled(Rect::new(10.0, 10.0, 100.0, 0.0), Rgba::hex(0xff0000)),
            Quad::filled(Rect::ZERO, Rgba::hex(0xff0000)),
        ],
    );

    assert_colour(canvas.pixel(&frame, 50, 50), background);
}

#[test]
fn the_instance_buffer_grows_past_its_starting_capacity() {
    let mut canvas = canvas();
    let fill = Rgba::hex(0x4a7d5f);

    let quads: Vec<Quad> = (0..400)
        .map(|i| {
            let x = (i % 20) as f32 * 12.0 + 4.0;
            let y = (i / 20) as f32 * 12.0 + 4.0;
            Quad::filled(Rect::new(x, y, 8.0, 8.0), fill)
        })
        .collect();

    let frame = canvas.render(Rgba::hex(0xffffff), &quads);

    assert_colour(canvas.pixel(&frame, 236, 236), fill);
    assert_colour(canvas.pixel(&frame, 128, 128), fill);
}

#[test]
fn frames_can_be_drawn_one_after_another() {
    let mut canvas = canvas();

    let first = canvas.render(Rgba::hex(0xff0000), &[]);
    let second = canvas.render(Rgba::hex(0x00ff00), &[]);

    assert_colour(canvas.pixel(&first, 128, 128), Rgba::hex(0xff0000));
    assert_colour(canvas.pixel(&second, 128, 128), Rgba::hex(0x00ff00));
}

#[test]
fn the_themes_own_colours_come_out_as_stated() {
    let mut canvas = canvas();
    let theme = Theme::light();

    let frame = canvas.render(
        theme.chrome.backdrop,
        &[
            Quad::filled(Rect::new(0.0, 0.0, 256.0, 40.0), theme.chrome.panel),
            Quad::filled(Rect::new(0.0, 40.0, 256.0, 180.0), theme.chrome.surface),
            Quad::filled(Rect::new(8.0, 200.0, 80.0, 24.0), theme.chrome.accent_solid).rounded(6.0),
        ],
    );

    assert_colour(canvas.pixel(&frame, 128, 20), theme.chrome.panel);
    assert_colour(canvas.pixel(&frame, 128, 120), theme.chrome.surface);
    assert_colour(canvas.pixel(&frame, 48, 212), theme.chrome.accent_solid);
    assert_colour(canvas.pixel(&frame, 200, 240), theme.chrome.backdrop);
}
