use crc_theme::{Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::settings::{self, BindingRow, Section, SettingsView, Target, Toggle};
use crc_ui::view::{self, CodeMetrics, EditorView};
use crc_ui::{Offscreen, Shell, ShellState};

fn window() -> Rect {
    Rect::new(0.0, 0.0, 1280.0, 800.0)
}

fn binding(command: &str, title: &str, keys: &str) -> BindingRow {
    BindingRow {
        command: command.to_string(),
        title: title.to_string(),
        keys: keys.to_string(),
        clash: None,
        changed: false,
    }
}

fn view() -> SettingsView {
    SettingsView {
        section: Section::Appearance,
        query: String::new(),
        toggles: vec![
            Toggle::new("dark", "Тёмная тема", "Светлый и тёмный набор", true),
            Toggle::new("rail", "Рейка действий", "Полоса у левого края", false),
            Toggle::new("explorer", "Проводник", "Дерево файлов", true),
            Toggle::new("tabs", "Вкладки", "Открытые файлы", true),
        ],
        bindings: vec![
            binding("save", "Сохранить", "Ctrl+S"),
            binding("palette", "Командная палитра", "Ctrl+K"),
            binding("theme", "Сменить тему", ""),
        ],
        capturing: None,
        hovered: None,
        scroll: 0,
    }
}

#[test]
fn the_panel_sits_in_the_middle_of_the_window() {
    let placed = settings::layout(window(), &view(), 1.0);

    let left = placed.panel.x - window().x;
    let right = window().right() - placed.panel.right();
    let top = placed.panel.y - window().y;
    let bottom = window().bottom() - placed.panel.bottom();

    assert!((left - right).abs() < 0.5, "off centre horizontally");
    assert!((top - bottom).abs() < 0.5, "off centre vertically");
}

#[test]
fn the_header_the_sidebar_and_the_body_carve_up_the_panel() {
    let placed = settings::layout(window(), &view(), 1.0);

    assert_eq!(placed.header.y, placed.panel.y);
    assert_eq!(placed.header.bottom(), placed.sidebar.y);
    assert_eq!(placed.sidebar.bottom(), placed.panel.bottom());
    assert_eq!(placed.sidebar.right(), placed.body.x);
    assert_eq!(placed.body.right(), placed.panel.right());
    assert!(placed.close.right() <= placed.header.right());
}

#[test]
fn every_section_gets_a_row_and_they_stack() {
    let placed = settings::layout(window(), &view(), 1.0);

    assert_eq!(placed.sections.len(), Section::ALL.len());
    for index in 1..placed.sections.len() {
        assert_eq!(
            placed.sections[index - 1].bottom(),
            placed.sections[index].y
        );
    }
    assert!(placed.sections.iter().all(|row| row.right() <= placed.sidebar.right()));
}

#[test]
fn the_rows_belong_to_the_section_on_show() {
    let mut state = view();
    let appearance = settings::layout(window(), &state, 1.0);
    assert_eq!(appearance.rows.len(), state.toggles.len());

    state.section = Section::Keys;
    let keys = settings::layout(window(), &state, 1.0);
    assert_eq!(keys.rows.len(), state.bindings.len());
}

#[test]
fn a_long_list_is_cut_to_what_fits_and_scrolling_moves_the_window() {
    let mut state = view();
    state.toggles = (0..40)
        .map(|index| Toggle::new(&format!("t{index}"), &format!("Пункт {index}"), "", false))
        .collect();

    let placed = settings::layout(window(), &state, 1.0);
    let fits = settings::visible_rows(&placed);
    assert!(fits > 0 && fits < 40, "the list should be clipped: {fits}");
    assert!(placed.rows.last().unwrap().bottom() <= placed.body.bottom());

    state.scroll = 5;
    let scrolled = settings::layout(window(), &state, 1.0);
    assert_eq!(scrolled.rows.len(), fits);
    assert_eq!(
        settings::target_at(&scrolled, &state, scrolled.rows[0].x + 8.0, scrolled.rows[0].y + 8.0),
        Some(Target::Toggle(5)),
        "the first drawn row is the sixth toggle"
    );
}

#[test]
fn the_panel_shrinks_to_fit_a_small_window() {
    let small = Rect::new(0.0, 0.0, 600.0, 420.0);
    let placed = settings::layout(small, &view(), 1.0);

    assert!(placed.panel.width <= small.width);
    assert!(placed.panel.height <= small.height);
    assert!(placed.panel.x >= small.x && placed.panel.bottom() <= small.bottom());
    assert!(!placed.rows.is_empty(), "at least one row still shows");
}

#[test]
fn it_scales_with_the_display() {
    let one = settings::layout(Rect::from_size(2560.0, 1600.0), &view(), 1.0);
    let two = settings::layout(Rect::from_size(2560.0, 1600.0), &view(), 2.0);

    assert!(two.panel.width > one.panel.width);
    assert!(two.rows[0].height > one.rows[0].height);
}

mod clashes {
    use super::*;

    #[test]
    fn a_chord_bound_twice_is_flagged_on_both_rows() {
        let mut rows = vec![
            binding("save", "Сохранить", "Ctrl+S"),
            binding("search", "Поиск", "Ctrl+S"),
            binding("palette", "Палитра", "Ctrl+K"),
        ];
        settings::mark_clashes(&mut rows);

        assert_eq!(rows[0].clash.as_deref(), Some("Поиск"));
        assert_eq!(rows[1].clash.as_deref(), Some("Сохранить"));
        assert_eq!(rows[2].clash, None, "a unique chord is fine");
    }

    #[test]
    fn case_does_not_hide_a_clash() {
        let mut rows = vec![
            binding("save", "Сохранить", "Ctrl+S"),
            binding("search", "Поиск", "ctrl+s"),
        ];
        settings::mark_clashes(&mut rows);

        assert!(rows[0].clash.is_some());
        assert!(rows[1].clash.is_some());
    }

    #[test]
    fn unbound_commands_never_clash_with_each_other() {
        let mut rows = vec![
            binding("theme", "Тема", ""),
            binding("zen", "Тихий режим", ""),
        ];
        settings::mark_clashes(&mut rows);

        assert!(rows.iter().all(|row| row.clash.is_none()));
    }

    #[test]
    fn a_flag_is_cleared_once_the_chord_moves() {
        let mut rows = vec![
            binding("save", "Сохранить", "Ctrl+S"),
            binding("search", "Поиск", "Ctrl+S"),
        ];
        settings::mark_clashes(&mut rows);
        assert!(rows[0].clash.is_some());

        rows[1].keys = "Ctrl+F".to_string();
        settings::mark_clashes(&mut rows);
        assert!(rows.iter().all(|row| row.clash.is_none()));
    }
}

mod clicking {
    use super::*;

    #[test]
    fn the_cross_closes_the_panel() {
        let state = view();
        let placed = settings::layout(window(), &state, 1.0);
        let close = placed.close;

        assert_eq!(
            settings::target_at(&placed, &state, close.x + 4.0, close.y + 4.0),
            Some(Target::Close)
        );
    }

    #[test]
    fn a_click_in_the_sidebar_names_the_section() {
        let state = view();
        let placed = settings::layout(window(), &state, 1.0);

        for index in 0..Section::ALL.len() {
            let row = placed.sections[index];
            assert_eq!(
                settings::target_at(&placed, &state, row.x + 10.0, row.y + 8.0),
                Some(Target::Section(index))
            );
        }
    }

    #[test]
    fn rows_name_a_toggle_or_a_binding_depending_on_the_section() {
        let mut state = view();
        let placed = settings::layout(window(), &state, 1.0);
        let row = placed.rows[2];
        assert_eq!(
            settings::target_at(&placed, &state, row.x + 8.0, row.y + 8.0),
            Some(Target::Toggle(2))
        );

        state.section = Section::Keys;
        let placed = settings::layout(window(), &state, 1.0);
        let row = placed.rows[1];
        assert_eq!(
            settings::target_at(&placed, &state, row.x + 8.0, row.y + 8.0),
            Some(Target::Binding(1))
        );
    }

    #[test]
    fn a_click_outside_the_panel_names_nothing() {
        let state = view();
        let placed = settings::layout(window(), &state, 1.0);

        assert_eq!(settings::target_at(&placed, &state, 4.0, 4.0), None);
        assert_eq!(
            settings::target_at(&placed, &state, placed.panel.right() + 20.0, placed.panel.y + 20.0),
            None
        );
    }

    #[test]
    fn the_header_is_not_a_button() {
        let state = view();
        let placed = settings::layout(window(), &state, 1.0);

        assert_eq!(
            settings::target_at(&placed, &state, placed.header.x + 30.0, placed.header.y + 20.0),
            None
        );
    }
}

mod drawing {
    use super::*;

    const WIDTH: u32 = 1100;
    const HEIGHT: u32 = 820;

    fn near(a: Rgba, b: Rgba) -> bool {
        a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
    }

    fn editor(settings: SettingsView) -> EditorView {
        EditorView {
            project: "crc-code".to_string(),
            focused: true,
            settings: Some(settings),
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
    fn the_panel_covers_the_editor_it_floats_over() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);
        let state = view();

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(state.clone()),
            CodeMetrics::default(),
        ));

        let placed = settings::layout(layout.window, &state, theme.scale);
        let inside = Rect::new(
            placed.body.x + 20.0,
            placed.body.bottom() - 20.0,
            60.0,
            10.0,
        );

        assert!(
            canvas.count_pixels(&pixels, inside, |c| near(c, theme.chrome.raised)) > 300,
            "the panel did not paint over the buffer"
        );
        assert_eq!(
            canvas.count_pixels(&pixels, inside, |c| near(c, theme.chrome.surface)),
            0,
            "the buffer shows through the panel"
        );
    }

    #[test]
    fn a_switch_that_is_on_is_painted_in_the_accent() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);
        let state = view();

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(state.clone()),
            CodeMetrics::default(),
        ));

        let placed = settings::layout(layout.window, &state, theme.scale);
        let accent = |c: Rgba| near(c, theme.chrome.accent_solid);

        let on = canvas.count_pixels(&pixels, placed.rows[0], accent);
        let off = canvas.count_pixels(&pixels, placed.rows[1], accent);

        assert!(on > 300, "the first switch is on but did not light up: {on}");
        assert!(
            off < on / 10,
            "the second switch is off and should stay quiet: {off}"
        );
    }

    #[test]
    fn a_row_listening_for_a_chord_is_washed_in_the_accent() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);

        let mut state = view();
        state.section = Section::Keys;
        let resting = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(state.clone()),
            CodeMetrics::default(),
        ));

        state.capturing = Some(1);
        let listening = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(state.clone()),
            CodeMetrics::default(),
        ));

        let placed = settings::layout(layout.window, &state, theme.scale);
        let row = placed.rows[1];
        let wash = |c: Rgba| near(c, theme.chrome.accent_wash);

        let quiet = canvas.count_pixels(&resting, row, wash);
        let lit = canvas.count_pixels(&listening, row, wash);

        assert!(
            lit > 2000 && lit > quiet * 10,
            "the row did not say it is listening: {quiet} at rest, {lit} listening"
        );
    }

    #[test]
    fn a_clash_is_drawn_in_the_danger_colour() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);

        let mut state = view();
        state.section = Section::Keys;
        let clean = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(state.clone()),
            CodeMetrics::default(),
        ));

        state.bindings[1].keys = state.bindings[0].keys.clone();
        settings::mark_clashes(&mut state.bindings);
        assert!(state.bindings[1].clash.is_some(), "the fixture should clash");

        let clashing = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(state.clone()),
            CodeMetrics::default(),
        ));

        let placed = settings::layout(layout.window, &state, theme.scale);
        let row = placed.rows[1];
        let danger = |c: Rgba| near(c, theme.chrome.danger);

        let quiet = canvas.count_pixels(&clean, row, danger);
        let loud = canvas.count_pixels(&clashing, row, danger);

        assert!(
            loud > 50 && loud > quiet + 40,
            "the conflict was not shown: {quiet} clean, {loud} clashing"
        );
    }

    #[test]
    fn an_unbound_command_says_so_instead_of_showing_an_empty_cap() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::light();
        let layout = shell(&theme);

        let mut state = view();
        state.section = Section::Keys;
        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(state.clone()),
            CodeMetrics::default(),
        ));

        let placed = settings::layout(layout.window, &state, theme.scale);
        let cap = Rect::new(
            placed.rows[2].right() - settings::KEYCAP * theme.scale,
            placed.rows[2].y,
            settings::KEYCAP * theme.scale,
            placed.rows[2].height,
        );

        assert!(
            canvas.count_pixels(&pixels, cap, |c| near(c, theme.chrome.text_faint)) > 20,
            "the empty binding drew no label"
        );
    }
}

mod overflow {
    use super::*;

    fn many(count: usize) -> SettingsView {
        let mut state = view();
        state.toggles = (0..count)
            .map(|index| Toggle::new(&format!("t{index}"), &format!("Пункт {index}"), "", false))
            .collect();
        state
    }

    #[test]
    fn a_short_list_shows_no_scrollbar() {
        let placed = settings::layout(window(), &view(), 1.0);
        assert!(placed.thumb.is_none(), "nothing is hidden, nothing to show");
    }

    #[test]
    fn a_long_list_grows_one_and_gives_it_room() {
        let short = settings::layout(window(), &view(), 1.0);
        let state = many(40);
        let placed = settings::layout(window(), &state, 1.0);

        let thumb = placed.thumb.expect("a list this long is scrollable");
        assert!(placed.body.contains(thumb.x, thumb.y));
        assert!(thumb.right() <= placed.body.right());
        assert!(
            placed.rows[0].width < short.rows[0].width,
            "the rows should step aside for the scrollbar"
        );
        assert!(placed.rows[0].right() <= thumb.x);
    }

    #[test]
    fn the_thumb_is_shorter_the_longer_the_list() {
        let ten = settings::layout(window(), &many(10), 1.0).thumb.unwrap();
        let hundred = settings::layout(window(), &many(100), 1.0).thumb.unwrap();

        assert!(hundred.height < ten.height);
    }

    #[test]
    fn the_thumb_travels_from_top_to_bottom_with_the_scroll() {
        let mut state = many(40);
        let top = settings::layout(window(), &state, 1.0).thumb.unwrap();

        let placed = settings::layout(window(), &state, 1.0);
        state.scroll = 40 - settings::visible_rows(&placed);
        let bottom = settings::layout(window(), &state, 1.0).thumb.unwrap();

        assert!(bottom.y > top.y, "the thumb never moved");
        assert!(
            bottom.bottom() <= placed.body.bottom(),
            "the thumb ran off the panel"
        );
    }
}

mod searching {
    use super::*;

    fn keys() -> SettingsView {
        let mut state = view();
        state.section = Section::Keys;
        state.bindings = vec![
            binding("save", "Сохранить файл", "Ctrl+S"),
            binding("close-tab", "Закрыть вкладку", "Ctrl+W"),
            binding("theme", "Переключить светлую и тёмную тему", "Ctrl+D"),
            binding("sidebar", "Показать или скрыть проводник", "Ctrl+B"),
        ];
        state
    }

    #[test]
    fn an_empty_query_shows_everything() {
        let state = keys();
        assert_eq!(state.shown(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn the_query_narrows_the_list_by_title() {
        let mut state = keys();
        state.query = "тему".to_string();
        assert_eq!(state.shown(), vec![2]);
    }

    #[test]
    fn the_query_also_matches_the_chord() {
        let mut state = keys();
        state.query = "ctrl+w".to_string();
        assert_eq!(state.shown(), vec![1]);
    }

    #[test]
    fn the_query_ignores_case_and_stray_spaces() {
        let mut state = keys();
        state.query = "  ЗАКРЫТЬ ".to_string();
        assert_eq!(state.shown(), vec![1]);
    }

    #[test]
    fn a_query_that_matches_nothing_leaves_no_rows_to_draw() {
        let mut state = keys();
        state.query = "нет такого".to_string();
        let placed = settings::layout(window(), &state, 1.0);

        assert!(state.shown().is_empty());
        assert!(placed.rows.is_empty());
        assert!(placed.thumb.is_none());
    }

    #[test]
    fn a_click_lands_on_the_command_under_the_cursor_not_its_place_in_the_list() {
        let mut state = keys();
        state.query = "скрыть".to_string();
        let placed = settings::layout(window(), &state, 1.0);
        let row = placed.rows[0];

        assert_eq!(
            settings::target_at(&placed, &state, row.x + 8.0, row.y + 8.0),
            Some(Target::Binding(3)),
            "the only match is the fourth command"
        );
    }

    #[test]
    fn the_search_field_and_the_reset_button_belong_to_the_keys_section() {
        let plain = settings::layout(window(), &view(), 1.0);
        assert!(plain.search.is_none() && plain.reset.is_none());

        let placed = settings::layout(window(), &keys(), 1.0);
        let field = placed.search.expect("a search field");
        let button = placed.reset.expect("a reset button");

        assert!(field.right() <= button.x, "the field and the button overlap");
        assert!(button.right() <= placed.body.right());
        assert!(field.bottom() <= placed.rows[0].y, "the list starts below");
    }

    #[test]
    fn the_search_row_costs_the_list_some_height() {
        let mut wide = keys();
        wide.bindings = (0..40)
            .map(|index| binding(&format!("c{index}"), &format!("Команда {index}"), "Ctrl+A"))
            .collect();

        let with_search = settings::layout(window(), &wide, 1.0);

        let mut without = wide.clone();
        without.section = Section::Appearance;
        without.toggles = (0..40)
            .map(|index| Toggle::new(&format!("t{index}"), &format!("Пункт {index}"), "", false))
            .collect();
        let plain = settings::layout(window(), &without, 1.0);

        assert!(with_search.rows.len() < plain.rows.len());
    }

    #[test]
    fn clicking_the_field_or_the_button_is_told_apart() {
        let state = keys();
        let placed = settings::layout(window(), &state, 1.0);
        let field = placed.search.unwrap();
        let button = placed.reset.unwrap();

        assert_eq!(
            settings::target_at(&placed, &state, field.x + 20.0, field.y + 10.0),
            Some(Target::Search)
        );
        assert_eq!(
            settings::target_at(&placed, &state, button.x + 20.0, button.y + 10.0),
            Some(Target::Reset)
        );
    }

    #[test]
    fn the_reset_button_knows_whether_anything_was_changed() {
        let mut state = keys();
        assert!(!state.touched());

        state.bindings[1].changed = true;
        assert!(state.touched());
    }
}
