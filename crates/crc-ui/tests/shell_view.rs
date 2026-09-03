use crc_syntax::{Language, SyntaxTree};
use crc_theme::{Density, Highlight, Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::{self, CodeMetrics, EditorView, FileEntry, Tab};
use crc_ui::{Offscreen, Shell, ShellState};

const WIDTH: u32 = 640;
const HEIGHT: u32 = 400;

fn source() -> String {
    (0..40)
        .map(|i| format!("const value{i} = compute({i})"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn view() -> EditorView {
    let text = source();
    let mut tree = SyntaxTree::new(Language::TypeScript).unwrap();
    tree.parse(&text).unwrap();
    let highlights = tree
        .highlights(&text)
        .into_iter()
        .map(|span| (span.range, span.highlight))
        .collect();

    EditorView {
        project: "crc-code".to_string(),
        branch: "main".to_string(),
        tabs: vec![Tab::new("Editor.tsx").active(), Tab::new("theme.css")],
        files: vec![
            FileEntry::dir("src", 0),
            FileEntry::file("Editor.tsx", 1).selected(),
            FileEntry::file("theme.css", 1).modified(),
        ],
        text,
        highlights,
        cursor_line: 2,
        cursor_column: 6,
        scroll_line: 0,
        language: "TypeScript".to_string(),
        problems: 2,
        focused: true,
        maximized: false,
        hovered_control: None,
        hovered_tab: None,
        palette: None,
        welcome: None,
        settings: None,
        selection: None,
        dirty: false,
    }
}

fn layout(theme: &Theme) -> Shell {
    Shell::compute(
        Rect::from_size(WIDTH as f32, HEIGHT as f32),
        theme,
        &ShellState::default(),
    )
}

fn near(a: Rgba, b: Rgba) -> bool {
    a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
}

#[test]
fn the_shell_paints_each_region_in_its_own_colour() {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let theme = Theme::light();
    let layout = layout(&theme);

    let frame = view::draw(&layout, &theme, &view(), CodeMetrics::default());
    let pixels = canvas.render_frame(&frame);

    let sample = |rect: Rect| {
        canvas.pixel(
            &pixels,
            (rect.x + rect.width / 2.0) as u32,
            (rect.y + rect.height / 2.0) as u32,
        )
    };

    let title_edge = canvas.pixel(&pixels, WIDTH - 20, (layout.titlebar.height / 2.0) as u32);
    assert!(
        near(title_edge, theme.chrome.panel),
        "title bar is {title_edge:?}"
    );
    let sidebar = layout.sidebar.expect("a sidebar");
    assert!(
        near(
            canvas.pixel(
                &pixels,
                (sidebar.x + 4.0) as u32,
                sidebar.bottom() as u32 - 4
            ),
            theme.chrome.panel
        ),
        "sidebar is not on its panel colour"
    );
    let statusbar = layout.statusbar.expect("a status bar");
    assert!(near(sample(statusbar), theme.chrome.panel));
}

#[test]
fn the_buffer_gets_ink_and_the_gutter_gets_numbers() {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let theme = Theme::light();
    let layout = layout(&theme);

    let frame = view::draw(&layout, &theme, &view(), CodeMetrics::default());
    let pixels = canvas.render_frame(&frame);

    let has_ink = |c: Rgba| !near(c, theme.chrome.surface);
    assert!(
        canvas.count_pixels(&pixels, layout.buffer, has_ink) > 200,
        "the code did not render"
    );
    assert!(
        canvas.count_pixels(&pixels, layout.gutter, has_ink) > 20,
        "the line numbers did not render"
    );
}

#[test]
fn the_active_tab_is_marked_with_the_accent() {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let theme = Theme::light();
    let layout = layout(&theme);

    let frame = view::draw(&layout, &theme, &view(), CodeMetrics::default());
    let pixels = canvas.render_frame(&frame);

    let accent = theme.chrome.accent;
    let is_accent = |c: Rgba| near(c, accent);
    assert!(
        canvas.count_pixels(&pixels, layout.tabs, is_accent) > 10,
        "no accent underline on the active tab"
    );
}

#[test]
fn the_caret_shows_up_on_the_cursor_line() {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let theme = Theme::light();
    let layout = layout(&theme);
    let metrics = CodeMetrics::default();

    let frame = view::draw(&layout, &theme, &view(), metrics);
    let pixels = canvas.render_frame(&frame);

    let caret_row = Rect::new(
        layout.buffer.x,
        layout.buffer.y + 2.0 * metrics.line_height,
        layout.buffer.width,
        metrics.line_height,
    );
    let is_caret = |c: Rgba| near(c, theme.syntax.caret);
    assert!(
        canvas.count_pixels(&pixels, caret_row, is_caret) > 3,
        "the caret is not on line 3"
    );
}

#[test]
fn zen_hands_the_window_to_the_code() {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let mut theme = Theme::light();
    theme.zen = true;
    let layout = layout(&theme);

    let frame = view::draw(&layout, &theme, &view(), CodeMetrics::default());
    let pixels = canvas.render_frame(&frame);

    assert!(layout.sidebar.is_none());
    let below_titlebar = Rect::new(
        0.0,
        layout.titlebar.bottom() + 2.0,
        WIDTH as f32,
        HEIGHT as f32 - layout.titlebar.height - 4.0,
    );
    let on_surface =
        canvas.count_pixels(&pixels, below_titlebar, |c| near(c, theme.chrome.surface));
    let area = (below_titlebar.width * below_titlebar.height) as usize;
    assert!(
        on_surface * 2 > area,
        "the buffer surface should dominate in zen, covered {on_surface} of {area}"
    );
}

#[test]
fn every_density_profile_draws_without_falling_over() {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");

    for density in [Density::Calm, Density::Balanced, Density::Dense] {
        let theme = Theme::light().with_density(density);
        let layout = layout(&theme);
        let frame = view::draw(&layout, &theme, &view(), CodeMetrics::default());
        let pixels = canvas.render_frame(&frame);

        assert!(
            canvas.count_pixels(&pixels, layout.buffer, |c| !near(c, theme.chrome.surface)) > 100,
            "{density:?} rendered an empty buffer"
        );
    }
}

#[test]
fn an_empty_view_still_draws_a_shell() {
    let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
    let theme = Theme::light();
    let layout = layout(&theme);

    let frame = view::draw(
        &layout,
        &theme,
        &EditorView::default(),
        CodeMetrics::default(),
    );
    let pixels = canvas.render_frame(&frame);

    assert!(near(
        canvas.pixel(&pixels, WIDTH / 2, layout.titlebar.height as u32 / 2),
        theme.chrome.panel
    ));
}

mod scrolling {
    use super::*;

    fn lines(count: usize) -> EditorView {
        EditorView {
            text: (0..count)
                .map(|i| format!("line{i}"))
                .collect::<Vec<_>>()
                .join("\n"),
            ..EditorView::default()
        }
    }

    #[test]
    fn the_top_of_the_file_starts_at_the_first_line() {
        let view = lines(10);
        let visible = view.visible(3);

        assert_eq!(visible.first_line, 0);
        assert_eq!(visible.text, "line0\nline1\nline2\n");
    }

    #[test]
    fn scrolling_moves_the_window_over_the_text() {
        let mut view = lines(10);
        view.scroll_line = 4;

        let visible = view.visible(3);

        assert_eq!(visible.first_line, 4);
        assert_eq!(visible.text, "line4\nline5\nline6\n");
    }

    #[test]
    fn asking_for_more_rows_than_there_are_lines_stops_at_the_end() {
        let view = lines(3);
        let visible = view.visible(50);

        assert_eq!(visible.text, "line0\nline1\nline2");
    }

    #[test]
    fn scrolling_past_the_end_yields_nothing_rather_than_panicking() {
        let mut view = lines(3);
        view.scroll_line = 99;

        let visible = view.visible(5);

        assert_eq!(visible.text, "");
        assert!(visible.spans.is_empty());
    }

    #[test]
    fn highlights_are_cut_to_the_visible_window_and_rebased() {
        let mut view = lines(10);
        view.scroll_line = 2;
        view.highlights = vec![
            (0..5, Highlight::Keyword),
            (12..17, Highlight::String),
            (200..210, Highlight::Comment),
        ];

        let visible = view.visible(2);

        assert_eq!(visible.text, "line2\nline3\n");
        assert_eq!(
            visible.spans,
            vec![(0..5, Highlight::String)],
            "only the span inside the window survives, moved to local offsets"
        );
    }

    #[test]
    fn a_span_straddling_the_edge_is_clipped_not_dropped() {
        let mut view = lines(10);
        view.scroll_line = 1;
        view.highlights = vec![(3..14, Highlight::Comment)];

        let visible = view.visible(2);

        assert_eq!(visible.spans.len(), 1);
        assert_eq!(visible.spans[0].0.start, 0);
        assert!(visible.spans[0].0.end <= visible.text.len());
    }
}

mod window_controls {
    use super::*;
    use crc_ui::{WindowControl, control_rect};

    fn dot(canvas: &Offscreen, pixels: &[u8], layout: &Shell, control: WindowControl) -> Rgba {
        let rect = control_rect(layout.titlebar, control);
        canvas.pixel(
            pixels,
            (rect.x + rect.width / 2.0) as u32,
            (rect.y + rect.height / 2.0) as u32,
        )
    }

    fn shoulder(canvas: &Offscreen, pixels: &[u8], layout: &Shell, control: WindowControl) -> Rgba {
        let rect = control_rect(layout.titlebar, control);
        canvas.pixel(
            pixels,
            (rect.x + rect.width / 2.0) as u32,
            (rect.y + 2.0) as u32,
        )
    }

    #[test]
    fn a_focused_window_shows_the_three_colours() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(),
            CodeMetrics::default(),
        ));

        assert!(near(
            dot(&canvas, &pixels, &layout, WindowControl::Close),
            theme.chrome.control_close
        ));
        assert!(near(
            dot(&canvas, &pixels, &layout, WindowControl::Minimize),
            theme.chrome.control_minimize
        ));
        assert!(near(
            dot(&canvas, &pixels, &layout, WindowControl::Maximize),
            theme.chrome.control_maximize
        ));
    }

    #[test]
    fn an_unfocused_window_greys_them_out() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);
        let mut state = view();
        state.focused = false;

        let pixels =
            canvas.render_frame(&view::draw(&layout, &theme, &state, CodeMetrics::default()));

        for control in WindowControl::ALL {
            assert!(
                near(
                    dot(&canvas, &pixels, &layout, control),
                    theme.chrome.control_idle
                ),
                "{control:?} stayed coloured while the window was not focused"
            );
        }
    }

    #[test]
    fn hovering_darkens_only_the_dot_under_the_pointer() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let plain = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(),
            CodeMetrics::default(),
        ));
        let mut hovered = view();
        hovered.hovered_control = Some(WindowControl::Close);
        let lit = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &hovered,
            CodeMetrics::default(),
        ));

        let before = shoulder(&canvas, &plain, &layout, WindowControl::Close);
        let after = shoulder(&canvas, &lit, &layout, WindowControl::Close);
        assert!(
            after.relative_luminance() < before.relative_luminance(),
            "hover did not darken the close button"
        );

        assert_eq!(
            shoulder(&canvas, &plain, &layout, WindowControl::Maximize),
            shoulder(&canvas, &lit, &layout, WindowControl::Maximize),
            "only the dot under the pointer changes colour"
        );

        assert_ne!(
            dot(&canvas, &plain, &layout, WindowControl::Maximize),
            dot(&canvas, &lit, &layout, WindowControl::Maximize),
            "but every dot shows its sign, the way a mac does"
        );
    }

    #[test]
    fn the_dark_theme_draws_the_shell_too() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = layout(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(),
            CodeMetrics::default(),
        ));

        assert!(near(
            canvas.pixel(&pixels, WIDTH - 20, (layout.titlebar.height / 2.0) as u32),
            theme.chrome.panel
        ));
        assert!(
            canvas.count_pixels(&pixels, layout.buffer, |c| !near(c, theme.chrome.surface)) > 200,
            "the code did not render on the dark theme"
        );
        assert!(near(
            dot(&canvas, &pixels, &layout, WindowControl::Close),
            theme.chrome.control_close
        ));
    }
}
