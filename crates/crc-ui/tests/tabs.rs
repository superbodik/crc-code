use crc_theme::{Rgba, Theme, TypeScale};
use crc_ui::geometry::Rect;
use crc_ui::view::tabs::{self, TabHit};
use crc_ui::view::{self, CodeMetrics, EditorView, Tab};
use crc_ui::{Offscreen, Shell, ShellState};

fn scale() -> TypeScale {
    TypeScale::default_scale()
}

fn bar() -> Rect {
    Rect::new(0.0, 40.0, 900.0, 34.0)
}

fn three() -> Vec<Tab> {
    vec![
        Tab::new("main.rs").active(),
        Tab::new("theme.css"),
        Tab::new("Editor.tsx").modified(),
    ]
}

#[test]
fn tabs_are_laid_out_left_to_right_without_gaps() {
    let rects = tabs::rects(bar(), &three(), &scale());

    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].x, bar().x);
    assert_eq!(rects[0].right(), rects[1].x);
    assert_eq!(rects[1].right(), rects[2].x);
}

#[test]
fn a_longer_name_makes_a_wider_tab() {
    let short = tabs::width(&Tab::new("a.rs"), &scale());
    let long = tabs::width(&Tab::new("a-much-longer-name.rs"), &scale());

    assert!(long > short);
}

#[test]
fn every_tab_leaves_room_for_its_close_button() {
    for tab in three() {
        let width = tabs::width(&tab, &scale());
        assert!(
            width > tabs::PADDING * 2.0 + tabs::CLOSE_SIZE,
            "{} is too narrow to hold a close button",
            tab.name
        );
    }
}

#[test]
fn tabs_that_do_not_fit_are_left_off() {
    let narrow = Rect::new(0.0, 40.0, 130.0, 34.0);
    let rects = tabs::rects(narrow, &three(), &scale());

    assert!(rects.len() < 3, "the bar cannot hold all three");
    for rect in &rects {
        assert!(rect.right() <= narrow.right());
    }
}

mod hitting {
    use super::*;

    fn hit(x: f32, y: f32) -> Option<TabHit> {
        tabs::hit(bar(), &three(), &scale(), x, y)
    }

    #[test]
    fn clicking_a_tab_selects_it() {
        let rects = tabs::rects(bar(), &three(), &scale());

        assert_eq!(hit(rects[0].x + 6.0, 50.0), Some(TabHit::Select(0)));
        assert_eq!(hit(rects[1].x + 6.0, 50.0), Some(TabHit::Select(1)));
        assert_eq!(hit(rects[2].x + 6.0, 50.0), Some(TabHit::Select(2)));
    }

    #[test]
    fn clicking_the_cross_closes_instead_of_selecting() {
        let rects = tabs::rects(bar(), &three(), &scale());
        let close = tabs::close_rect(rects[1]);

        assert_eq!(
            hit(close.x + close.width / 2.0, close.y + close.height / 2.0),
            Some(TabHit::Close(1))
        );
    }

    #[test]
    fn the_close_target_is_forgiving_but_not_the_whole_tab() {
        let rects = tabs::rects(bar(), &three(), &scale());
        let close = tabs::close_rect(rects[0]);

        assert_eq!(
            hit(close.x - 2.0, close.y + close.height / 2.0),
            Some(TabHit::Close(0)),
            "a near miss still closes"
        );
        assert_eq!(
            hit(rects[0].x + tabs::PADDING, 50.0),
            Some(TabHit::Select(0)),
            "the label still selects"
        );
    }

    #[test]
    fn clicking_past_the_last_tab_hits_nothing() {
        let rects = tabs::rects(bar(), &three(), &scale());
        assert_eq!(hit(rects[2].right() + 40.0, 50.0), None);
    }

    #[test]
    fn clicking_outside_the_bar_hits_nothing() {
        assert_eq!(hit(20.0, 300.0), None);
        assert_eq!(hit(20.0, 10.0), None);
    }

    #[test]
    fn an_empty_bar_hits_nothing() {
        assert_eq!(tabs::hit(bar(), &[], &scale(), 20.0, 50.0), None);
    }
}

mod drawing {
    use super::*;

    const WIDTH: u32 = 900;
    const HEIGHT: u32 = 400;

    fn near(a: Rgba, b: Rgba) -> bool {
        a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
    }

    fn view(tabs: Vec<Tab>, hovered: Option<usize>) -> EditorView {
        EditorView {
            project: "crc-code".to_string(),
            tabs,
            text: "fn main() {}\n".to_string(),
            focused: true,
            hovered_tab: hovered,
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
    fn the_active_tab_reads_as_part_of_the_buffer() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(three(), None),
            CodeMetrics::default(),
        ));

        let rects = tabs::rects(layout.tabs, &three(), &theme.type_scale);
        let on_active = canvas.pixel(
            &pixels,
            (rects[0].x + 4.0) as u32,
            (rects[0].y + 20.0) as u32,
        );
        let on_idle = canvas.pixel(
            &pixels,
            (rects[1].x + 4.0) as u32,
            (rects[1].y + 20.0) as u32,
        );

        assert!(near(on_active, theme.chrome.surface));
        assert!(near(on_idle, theme.chrome.panel));
    }

    #[test]
    fn an_unsaved_tab_shows_a_dot_until_the_pointer_arrives() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);
        let is_warning = |c: Rgba| near(c, theme.chrome.warning);

        let resting = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(three(), None),
            CodeMetrics::default(),
        ));
        let hovered = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(three(), Some(2)),
            CodeMetrics::default(),
        ));

        assert!(
            canvas.count_pixels(&resting, layout.tabs, is_warning) > 5,
            "the unsaved dot is missing"
        );
        assert_eq!(
            canvas.count_pixels(&hovered, layout.tabs, is_warning),
            0,
            "hovering should swap the dot for the close cross"
        );
    }

    #[test]
    fn hovering_lifts_an_idle_tab() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);
        let rects = tabs::rects(layout.tabs, &three(), &theme.type_scale);

        let plain = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(three(), None),
            CodeMetrics::default(),
        ));
        let lit = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(three(), Some(1)),
            CodeMetrics::default(),
        ));

        let at = ((rects[1].x + 4.0) as u32, (rects[1].y + 20.0) as u32);
        assert!(near(canvas.pixel(&plain, at.0, at.1), theme.chrome.panel));
        assert!(near(canvas.pixel(&lit, at.0, at.1), theme.chrome.hover));
    }

    #[test]
    fn a_single_tab_still_draws() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(vec![Tab::new("only.rs").active()], None),
            CodeMetrics::default(),
        ));

        assert!(
            canvas.count_pixels(&pixels, layout.tabs, |c| near(c, theme.chrome.accent)) > 10,
            "the active marker is missing"
        );
    }

    #[test]
    fn no_open_files_leaves_an_empty_bar() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &view(Vec::new(), None),
            CodeMetrics::default(),
        ));

        assert_eq!(
            canvas.count_pixels(&pixels, layout.tabs, |c| near(c, theme.chrome.accent)),
            0
        );
    }
}
