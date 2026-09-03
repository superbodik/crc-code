use crc_text::Point;
use crc_theme::{Density, Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::hit::explorer_header_height;
use crc_ui::view::{self, CodeMetrics, EditorView, FileEntry, Tab};
use crc_ui::{Band, Offscreen, Shell, ShellState, bands, buffer_point, explorer_row};

fn metrics() -> CodeMetrics {
    CodeMetrics {
        char_width: 8.0,
        line_height: 20.0,
    }
}

mod selection_bands {
    use super::*;

    #[test]
    fn a_selection_inside_one_line_is_one_band() {
        let out = bands("hello world", &(2..7));

        assert_eq!(
            out,
            vec![Band {
                row: 0,
                start_column: 2,
                end_column: 7,
                to_line_end: false,
            }]
        );
    }

    #[test]
    fn a_selection_across_lines_becomes_one_band_per_line() {
        let out = bands("ab\ncd\nef", &(1..7));

        assert_eq!(out.len(), 3);
        assert_eq!(out[0].row, 0);
        assert_eq!((out[0].start_column, out[0].end_column), (1, 2));
        assert!(out[0].to_line_end, "the first line runs to its break");
        assert_eq!((out[1].start_column, out[1].end_column), (0, 2));
        assert_eq!((out[2].start_column, out[2].end_column), (0, 1));
        assert!(!out[2].to_line_end);
    }

    #[test]
    fn selecting_a_line_break_leaves_a_mark_past_the_text() {
        let out = bands("ab\ncd", &(0..3));

        assert_eq!(out.len(), 1);
        assert!(
            out[0].to_line_end,
            "the newline is selected, so the band reaches past the last character"
        );
    }

    #[test]
    fn an_empty_selection_draws_nothing() {
        assert!(bands("hello", &(3..3)).is_empty());
        assert!(bands("", &(0..0)).is_empty());
    }

    #[test]
    fn columns_count_characters_not_bytes() {
        let out = bands("привет мир", &(0..12));

        assert_eq!(out.len(), 1);
        assert_eq!(
            (out[0].start_column, out[0].end_column),
            (0, 6),
            "six two-byte letters are six columns"
        );
    }

    #[test]
    fn a_selection_past_the_end_is_clamped() {
        let out = bands("abc", &(1..999));

        assert_eq!(out.len(), 1);
        assert_eq!((out[0].start_column, out[0].end_column), (1, 3));
    }

    #[test]
    fn lines_the_selection_misses_get_no_band() {
        let out = bands("aaa\nbbb\nccc", &(4..7));

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].row, 1, "only the middle line");
    }
}

mod pointing {
    use super::*;

    fn buffer() -> Rect {
        Rect::new(100.0, 50.0, 400.0, 200.0)
    }

    #[test]
    fn a_click_maps_to_the_row_and_column_under_it() {
        let point = buffer_point(buffer(), metrics(), 0, 100.0 + 8.0 * 3.0, 50.0 + 20.0 * 2.0);
        assert_eq!(point, Point::new(2, 3));
    }

    #[test]
    fn the_scroll_offset_is_added_to_the_row() {
        let point = buffer_point(buffer(), metrics(), 40, 100.0, 50.0 + 20.0);
        assert_eq!(point, Point::new(41, 0));
    }

    #[test]
    fn a_click_lands_on_the_nearer_character_boundary() {
        let left = buffer_point(buffer(), metrics(), 0, 100.0 + 3.0, 60.0);
        assert_eq!(left.column, 0, "a third of the way in stays put");

        let right = buffer_point(buffer(), metrics(), 0, 100.0 + 6.0, 60.0);
        assert_eq!(right.column, 1, "past the middle moves to the next");
    }

    #[test]
    fn a_click_left_of_the_buffer_does_not_go_negative() {
        let point = buffer_point(buffer(), metrics(), 0, 0.0, 0.0);
        assert_eq!(point, Point::new(0, 0));
    }

    #[test]
    fn a_row_in_the_explorer_maps_to_its_entry() {
        let metrics = Density::Balanced.metrics();
        let sidebar = Rect::new(44.0, 40.0, 240.0, 700.0);
        let top = sidebar.y + explorer_header_height(&metrics);

        assert_eq!(explorer_row(sidebar, &metrics, top + 1.0), Some(0));
        assert_eq!(
            explorer_row(sidebar, &metrics, top + metrics.row_height + 1.0),
            Some(1)
        );
    }

    #[test]
    fn the_explorer_header_is_not_a_row() {
        let metrics = Density::Balanced.metrics();
        let sidebar = Rect::new(44.0, 40.0, 240.0, 700.0);

        assert_eq!(explorer_row(sidebar, &metrics, sidebar.y + 2.0), None);
        assert_eq!(
            explorer_row(sidebar, &metrics, sidebar.bottom() + 5.0),
            None
        );
    }
}

mod drawing {
    use super::*;

    const WIDTH: u32 = 640;
    const HEIGHT: u32 = 400;

    fn near(a: Rgba, b: Rgba) -> bool {
        a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
    }

    fn view(selection: Option<std::ops::Range<usize>>) -> EditorView {
        EditorView {
            project: "crc-code".to_string(),
            tabs: vec![Tab::new("main.rs").active()],
            files: vec![FileEntry::file("main.rs", 0).selected()],
            text: "fn main() {\n    let value = 1;\n    println!();\n}\n".to_string(),
            selection,
            focused: true,
            ..EditorView::default()
        }
    }

    fn layout(theme: &Theme) -> Shell {
        Shell::compute(
            Rect::from_size(WIDTH as f32, HEIGHT as f32),
            theme,
            &ShellState::default(),
        )
    }

    #[test]
    fn a_selection_paints_a_band_behind_the_text() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let plain = canvas.render_frame(&view::draw(&layout, &theme, &view(None), metrics()));
        let selected =
            canvas.render_frame(&view::draw(&layout, &theme, &view(Some(16..29)), metrics()));

        let is_selection = |c: Rgba| near(c, theme.syntax.selection);
        assert_eq!(
            canvas.count_pixels(&plain, layout.buffer, is_selection),
            0,
            "nothing is selected to begin with"
        );
        assert!(
            canvas.count_pixels(&selected, layout.buffer, is_selection) > 100,
            "the selection band did not render"
        );
    }

    #[test]
    fn the_band_sits_on_the_line_it_belongs_to() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);
        let metrics = metrics();

        let pixels =
            canvas.render_frame(&view::draw(&layout, &theme, &view(Some(16..29)), metrics));

        let is_selection = |c: Rgba| near(c, theme.syntax.selection);
        let row = |index: usize| {
            Rect::new(
                layout.buffer.x,
                layout.buffer.y + index as f32 * metrics.line_height,
                layout.buffer.width,
                metrics.line_height,
            )
        };

        assert!(canvas.count_pixels(&pixels, row(1), is_selection) > 50);
        assert_eq!(canvas.count_pixels(&pixels, row(0), is_selection), 0);
        assert_eq!(canvas.count_pixels(&pixels, row(3), is_selection), 0);
    }

    #[test]
    fn an_unsaved_file_is_marked_on_its_tab() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let mut dirty = view(None);
        dirty.dirty = true;

        let clean = canvas.render_frame(&view::draw(&layout, &theme, &view(None), metrics()));
        let marked = canvas.render_frame(&view::draw(&layout, &theme, &dirty, metrics()));

        let is_warning = |c: Rgba| near(c, theme.chrome.warning);
        assert_eq!(canvas.count_pixels(&clean, layout.tabs, is_warning), 0);
        assert!(
            canvas.count_pixels(&marked, layout.tabs, is_warning) > 5,
            "no unsaved marker on the tab"
        );
    }
}
