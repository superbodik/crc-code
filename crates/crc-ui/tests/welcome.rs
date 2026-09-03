use crc_theme::{Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::welcome::{self, RecentEntry, Target, WelcomeView};
use crc_ui::view::{self, CodeMetrics, EditorView};
use crc_ui::{Offscreen, Shell, ShellState};

fn window() -> Rect {
    Rect::new(0.0, 40.0, 1280.0, 760.0)
}

fn entry(name: &str) -> RecentEntry {
    RecentEntry {
        name: name.to_string(),
        path: format!("d:/Project/{name}"),
        when: "вчера".to_string(),
    }
}

fn view(count: usize) -> WelcomeView {
    WelcomeView {
        recent: (0..count).map(|i| entry(&format!("project{i}"))).collect(),
        hints: vec![
            ("Ctrl+K".to_string(), "Командная палитра".to_string()),
            ("Ctrl+O".to_string(), "Открыть проект".to_string()),
        ],
        hovered: None,
    }
}

#[test]
fn the_screen_is_centred_in_the_window() {
    let state = view(3);
    let placed = welcome::layout(window(), &state, 1.0);

    let left = placed.title.x - window().x;
    let right = window().right() - placed.title.right();
    assert!((left - right).abs() < 0.5, "off centre");
}

#[test]
fn everything_stacks_downwards_without_overlapping() {
    let state = view(3);
    let placed = welcome::layout(window(), &state, 1.0);

    assert!(placed.mark.bottom() <= placed.title.y);
    assert!(placed.title.bottom() <= placed.tagline.y);
    assert!(placed.tagline.bottom() <= placed.recent_heading.y);
    assert!(placed.recent_heading.bottom() <= placed.recent[0].y);
    assert!(placed.recent.last().unwrap().bottom() <= placed.open_folder.y);
    assert!(placed.open_folder.bottom() <= placed.hints[0].y);
}

#[test]
fn recent_rows_are_evenly_stacked() {
    let state = view(4);
    let placed = welcome::layout(window(), &state, 1.0);

    assert_eq!(placed.recent.len(), 4);
    for index in 1..4 {
        assert_eq!(placed.recent[index - 1].bottom(), placed.recent[index].y);
    }
}

#[test]
fn only_a_handful_of_projects_are_offered() {
    let state = view(30);
    let placed = welcome::layout(window(), &state, 1.0);

    assert_eq!(placed.recent.len(), welcome::MAX_RECENT);
}

#[test]
fn a_first_run_shows_the_button_without_a_recent_list() {
    let state = WelcomeView {
        recent: Vec::new(),
        hints: Vec::new(),
        hovered: None,
    };
    let placed = welcome::layout(window(), &state, 1.0);

    assert!(placed.recent.is_empty());
    assert!(placed.hints.is_empty());
    assert!(
        !placed.open_folder.is_empty(),
        "you can still open a folder"
    );
}

#[test]
fn the_screen_stays_inside_a_small_window() {
    let small = Rect::new(0.0, 40.0, 520.0, 400.0);
    let state = view(5);
    let placed = welcome::layout(small, &state, 1.0);

    assert!(placed.title.right() <= small.right());
    assert!(placed.title.x >= small.x);
}

#[test]
fn it_scales_with_the_display() {
    let state = view(3);
    let one = welcome::layout(window(), &state, 1.0);
    let two = welcome::layout(window(), &state, 2.0);

    assert!(two.open_folder.height > one.open_folder.height);
    assert!(two.mark.width > one.mark.width);
}

mod clicking {
    use super::*;

    #[test]
    fn a_click_on_a_project_names_it() {
        let state = view(3);
        let placed = welcome::layout(window(), &state, 1.0);

        for index in 0..3 {
            let row = placed.recent[index];
            assert_eq!(
                welcome::target_at(&placed, row.x + 20.0, row.y + 10.0),
                Some(Target::Recent(index))
            );
        }
    }

    #[test]
    fn a_click_on_the_button_opens_a_folder() {
        let state = view(2);
        let placed = welcome::layout(window(), &state, 1.0);
        let button = placed.open_folder;

        assert_eq!(
            welcome::target_at(&placed, button.x + 40.0, button.y + 10.0),
            Some(Target::OpenFolder)
        );
    }

    #[test]
    fn a_click_on_empty_space_names_nothing() {
        let state = view(2);
        let placed = welcome::layout(window(), &state, 1.0);

        assert_eq!(welcome::target_at(&placed, 5.0, 45.0), None);
        assert_eq!(
            welcome::target_at(&placed, placed.tagline.x + 10.0, placed.tagline.y + 4.0),
            None,
            "the tagline is not a button"
        );
    }
}

mod drawing {
    use super::*;

    const WIDTH: u32 = 900;
    const HEIGHT: u32 = 700;

    fn near(a: Rgba, b: Rgba) -> bool {
        a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
    }

    fn editor(welcome: Option<WelcomeView>) -> EditorView {
        EditorView {
            project: "crc-code".to_string(),
            focused: true,
            welcome,
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
    fn the_welcome_screen_replaces_the_shell_but_keeps_the_title_bar() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(Some(view(3))),
            CodeMetrics::default(),
        ));

        assert!(
            near(
                canvas.pixel(&pixels, WIDTH - 20, (layout.titlebar.height / 2.0) as u32),
                theme.chrome.panel
            ),
            "the title bar is still there to drag the window by"
        );

        let sidebar = layout.sidebar.expect("a sidebar in the normal shell");
        assert!(
            near(
                canvas.pixel(
                    &pixels,
                    (sidebar.x + 4.0) as u32,
                    sidebar.bottom() as u32 - 8
                ),
                theme.chrome.surface
            ),
            "the explorer is not drawn behind the welcome screen"
        );
    }

    #[test]
    fn the_open_button_is_painted_in_the_accent() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);
        let state = view(3);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(Some(state.clone())),
            CodeMetrics::default(),
        ));

        let window = Rect::new(
            layout.window.x,
            layout.titlebar.bottom(),
            layout.window.width,
            layout.window.bottom() - layout.titlebar.bottom(),
        );
        let placed = welcome::layout(window, &state, theme.scale);

        assert!(
            canvas.count_pixels(&pixels, placed.open_folder, |c| near(
                c,
                theme.chrome.accent_solid
            )) > 200,
            "the button did not paint"
        );
    }

    #[test]
    fn hovering_a_project_lifts_its_row() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = layout(&theme);

        let plain = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(Some(view(3))),
            CodeMetrics::default(),
        ));

        let mut lit_state = view(3);
        lit_state.hovered = Some(Target::Recent(1));
        let lit = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(Some(lit_state)),
            CodeMetrics::default(),
        ));

        let window = Rect::new(
            layout.window.x,
            layout.titlebar.bottom(),
            layout.window.width,
            layout.window.bottom() - layout.titlebar.bottom(),
        );
        let row = welcome::layout(window, &view(3), theme.scale).recent[1];

        let is_hover = |c: Rgba| near(c, theme.chrome.hover);
        let resting = canvas.count_pixels(&plain, row, is_hover);
        let lifted = canvas.count_pixels(&lit, row, is_hover);

        assert!(
            lifted > 2000 && lifted > resting * 10,
            "the row did not light up: {resting} at rest, {lifted} hovered"
        );
    }

    #[test]
    fn a_first_run_still_draws() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = layout(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(Some(WelcomeView::default())),
            CodeMetrics::default(),
        ));

        assert!(
            canvas.count_pixels(&pixels, Rect::from_size(WIDTH as f32, HEIGHT as f32), |c| {
                near(c, theme.chrome.accent_solid)
            }) > 100,
            "with no history there is still a way in"
        );
    }
}
