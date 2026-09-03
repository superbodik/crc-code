use crc_ui::geometry::Rect;
use crc_ui::view::controls::{CONTROL_DIAMETER, RESIZE_MARGIN};
use crc_ui::{Edge, WindowControl, control_at, control_rect, is_drag_handle, resize_edge};

fn titlebar() -> Rect {
    Rect::new(0.0, 0.0, 1440.0, 40.0)
}

fn window() -> Rect {
    Rect::from_size(1440.0, 900.0)
}

fn centre(rect: Rect) -> (f32, f32) {
    (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
}

#[test]
fn the_controls_sit_in_mac_order_from_the_left() {
    let bar = titlebar();
    let close = control_rect(bar, WindowControl::Close);
    let minimize = control_rect(bar, WindowControl::Minimize);
    let maximize = control_rect(bar, WindowControl::Maximize);

    assert!(close.x < minimize.x);
    assert!(minimize.x < maximize.x);
    assert_eq!(close.width, CONTROL_DIAMETER);
    assert_eq!(close.height, CONTROL_DIAMETER);
}

#[test]
fn the_controls_are_centred_in_the_title_bar() {
    let bar = Rect::new(0.0, 0.0, 800.0, 40.0);

    for control in WindowControl::ALL {
        let rect = control_rect(bar, control);
        let above = rect.y - bar.y;
        let below = bar.bottom() - rect.bottom();
        assert!(
            (above - below).abs() < 0.01,
            "{control:?} is off centre: {above} above, {below} below"
        );
    }
}

#[test]
fn a_click_on_each_control_finds_it() {
    let bar = titlebar();

    for control in WindowControl::ALL {
        let (x, y) = centre(control_rect(bar, control));
        assert_eq!(control_at(bar, x, y), Some(control));
    }
}

#[test]
fn the_hit_area_is_forgiving_around_the_dot() {
    let bar = titlebar();
    let close = control_rect(bar, WindowControl::Close);

    assert_eq!(
        control_at(bar, close.x - 2.0, close.y - 2.0),
        Some(WindowControl::Close),
        "a near miss still counts, the dots are small"
    );
}

#[test]
fn the_gap_between_controls_is_not_a_control() {
    let bar = titlebar();
    let close = control_rect(bar, WindowControl::Close);
    let minimize = control_rect(bar, WindowControl::Minimize);
    let gap = (close.right() + minimize.x) / 2.0;

    assert_eq!(control_at(bar, gap, 20.0), None);
}

#[test]
fn nothing_outside_the_title_bar_is_a_control() {
    let bar = titlebar();
    let (x, _) = centre(control_rect(bar, WindowControl::Close));

    assert_eq!(control_at(bar, x, 200.0), None, "below the bar");
    assert_eq!(control_at(bar, x, -5.0), None, "above the window");
    assert_eq!(control_at(bar, -20.0, 20.0), None, "left of the window");
}

#[test]
fn the_empty_stretch_of_the_title_bar_drags_the_window() {
    let bar = titlebar();

    assert!(is_drag_handle(bar, 700.0, 20.0));
    assert!(is_drag_handle(bar, 1400.0, 5.0));
}

#[test]
fn a_control_is_not_a_drag_handle() {
    let bar = titlebar();

    for control in WindowControl::ALL {
        let (x, y) = centre(control_rect(bar, control));
        assert!(
            !is_drag_handle(bar, x, y),
            "{control:?} would drag instead of act"
        );
    }
}

#[test]
fn the_buffer_is_not_a_drag_handle() {
    assert!(!is_drag_handle(titlebar(), 700.0, 400.0));
}

mod edges {
    use super::*;

    fn edge_at(x: f32, y: f32) -> Option<Edge> {
        resize_edge(window(), x, y, RESIZE_MARGIN)
    }

    #[test]
    fn each_corner_reports_its_own_diagonal() {
        assert_eq!(edge_at(1.0, 1.0), Some(Edge::TopLeft));
        assert_eq!(edge_at(1439.0, 1.0), Some(Edge::TopRight));
        assert_eq!(edge_at(1.0, 899.0), Some(Edge::BottomLeft));
        assert_eq!(edge_at(1439.0, 899.0), Some(Edge::BottomRight));
    }

    #[test]
    fn each_side_reports_its_own_axis() {
        assert_eq!(edge_at(700.0, 1.0), Some(Edge::Top));
        assert_eq!(edge_at(700.0, 899.0), Some(Edge::Bottom));
        assert_eq!(edge_at(1.0, 450.0), Some(Edge::Left));
        assert_eq!(edge_at(1439.0, 450.0), Some(Edge::Right));
    }

    #[test]
    fn the_middle_of_the_window_is_not_a_resize_grip() {
        assert_eq!(edge_at(700.0, 450.0), None);
        assert_eq!(edge_at(20.0, 20.0), None, "just inside the margin");
    }

    #[test]
    fn a_point_outside_the_window_grips_nothing() {
        assert_eq!(edge_at(-1.0, 450.0), None);
        assert_eq!(edge_at(700.0, 901.0), None);
    }

    #[test]
    fn a_wider_margin_reaches_further_in() {
        assert_eq!(resize_edge(window(), 700.0, 10.0, RESIZE_MARGIN), None);
        assert_eq!(
            resize_edge(window(), 700.0, 10.0, 20.0),
            Some(Edge::Top),
            "the grip scales with the display"
        );
    }

    #[test]
    fn the_title_bar_still_resizes_along_its_top_edge() {
        let bar = titlebar();
        assert!(is_drag_handle(bar, 700.0, 2.0));
        assert_eq!(
            edge_at(700.0, 2.0),
            Some(Edge::Top),
            "the edge is checked first, so the top strip resizes rather than drags"
        );
    }
}

mod glyphs {
    use super::*;
    use crc_theme::{Rgba, Theme};
    use crc_ui::view::{self, CodeMetrics, EditorView};
    use crc_ui::{Offscreen, Shell, ShellState};

    const WIDTH: u32 = 900;
    const HEIGHT: u32 = 600;

    fn editor(hovered: Option<WindowControl>) -> EditorView {
        EditorView {
            project: "crc-code".to_string(),
            focused: true,
            hovered_control: hovered,
            ..EditorView::default()
        }
    }

    fn shell(theme: &Theme) -> Shell {
        Shell::compute(
            Rect::from_size(WIDTH as f32, HEIGHT as f32),
            theme,
            &ShellState::default(),
        )
    }

    #[test]
    fn every_control_names_itself() {
        assert_eq!(WindowControl::Close.glyph(), "\u{00d7}");
        assert_eq!(WindowControl::Minimize.glyph(), "\u{2013}");
        assert_eq!(WindowControl::Maximize.glyph(), "+");
    }

    #[test]
    fn hovering_one_control_marks_all_three_the_way_a_mac_does() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);

        let resting = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(None),
            CodeMetrics::default(),
        ));
        let hovered = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(Some(WindowControl::Close)),
            CodeMetrics::default(),
        ));

        for control in WindowControl::ALL {
            let rect = control_rect(layout.titlebar, control).inset(2.0);
            let fill = match control {
                WindowControl::Close => theme.chrome.control_close,
                WindowControl::Minimize => theme.chrome.control_minimize,
                WindowControl::Maximize => theme.chrome.control_maximize,
            };
            let floor = fill.relative_luminance() * 0.25;
            let ink = |c: Rgba| c.relative_luminance() < floor;

            assert_eq!(
                canvas.count_pixels(&resting, rect, ink),
                0,
                "{control:?} should be a plain circle at rest"
            );
            let marks = canvas.count_pixels(&hovered, rect, ink);
            let softer = canvas.count_pixels(&hovered, rect, |c: Rgba| {
                c.relative_luminance() < fill.relative_luminance() * 0.7
            });
            assert!(
                marks > 3,
                "{control:?}: {marks} ink pixels, {softer} merely darker"
            );
        }
    }

    #[test]
    fn an_unfocused_window_keeps_its_signs_to_itself() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &EditorView {
                focused: false,
                hovered_control: Some(WindowControl::Close),
                ..editor(None)
            },
            CodeMetrics::default(),
        ));

        let rect = control_rect(layout.titlebar, WindowControl::Close).inset(2.0);
        let floor = theme.chrome.control_idle.relative_luminance() * 0.25;

        assert_eq!(
            canvas.count_pixels(&pixels, rect, |c: Rgba| c.relative_luminance() < floor),
            0,
            "a background window shows grey dots and nothing else"
        );
    }
}
