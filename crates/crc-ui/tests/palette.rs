use crc_ui::geometry::Rect;
use crc_ui::view::palette::{self, Action, PaletteView};

fn actions() -> Vec<Action> {
    vec![
        Action::new("save", "Сохранить файл", "Файл").hint("Ctrl+S"),
        Action::new("theme", "Переключить светлую и тёмную тему", "Вид").hint("Ctrl+D"),
        Action::new("sidebar", "Показать или скрыть проводник", "Вид"),
        Action::new("zen", "Zen — оставить только код", "Вид"),
        Action::new("close", "Закрыть вкладку", "Файл"),
    ]
}

fn ids(rows: &[palette::Row]) -> Vec<&str> {
    rows.iter().map(|row| row.id).collect()
}

#[test]
fn an_empty_query_offers_everything() {
    let rows = palette::filter(&actions(), "");
    assert_eq!(rows.len(), 5);
    assert!(rows.iter().all(|row| row.matched.is_empty()));
}

#[test]
fn typing_narrows_to_what_matches() {
    let rows = palette::filter(&actions(), "закр");

    assert_eq!(
        ids(&rows)[0],
        "close",
        "the word itself outranks a scattered match"
    );
    assert!(rows.len() < actions().len(), "the list did narrow");
    assert!(
        !ids(&rows).contains(&"zen"),
        "titles without those letters are gone"
    );
}

#[test]
fn letters_may_be_scattered_through_the_title() {
    let rows = palette::filter(&actions(), "звкл");
    assert_eq!(ids(&rows), vec!["close"], "З-а-В-кладку, not a substring");
}

#[test]
fn the_matched_letters_come_back_for_highlighting() {
    let rows = palette::filter(&actions(), "zen");
    let row = rows.first().expect("a match");

    let lit: String = row
        .matched
        .iter()
        .map(|range| &row.title[range.clone()])
        .collect();
    assert_eq!(lit, "Zen");
}

#[test]
fn a_run_of_letters_is_one_range_not_three() {
    let rows = palette::filter(&actions(), "zen");
    assert_eq!(rows[0].matched.len(), 1);
}

#[test]
fn a_match_at_the_start_beats_one_in_the_middle() {
    let list = vec![
        Action::new("late", "Открыть тему", "a"),
        Action::new("early", "Тема оформления", "a"),
    ];
    let rows = palette::filter(&list, "тема");

    assert_eq!(ids(&rows)[0], "early");
}

#[test]
fn a_shorter_title_wins_a_tie() {
    let list = vec![
        Action::new("long", "Сохранить файл и закрыть вкладку потом", "a"),
        Action::new("short", "Сохранить файл", "a"),
    ];
    let rows = palette::filter(&list, "сохранить");

    assert_eq!(ids(&rows)[0], "short");
}

#[test]
fn matching_ignores_case() {
    assert_eq!(ids(&palette::filter(&actions(), "ZEN")), vec!["zen"]);
    assert_eq!(ids(&palette::filter(&actions(), "zEn")), vec!["zen"]);
}

#[test]
fn nothing_matching_gives_nothing() {
    assert!(palette::filter(&actions(), "щщщщ").is_empty());
}

#[test]
fn the_list_is_capped_so_the_panel_cannot_grow_forever() {
    let many: Vec<Action> = (0..40)
        .map(|i| Action::new("x", format!("Команда номер {i}"), "a"))
        .collect();

    assert_eq!(palette::filter(&many, "команда").len(), palette::MAX_ROWS);
}

mod selecting {
    use super::*;

    fn view() -> PaletteView {
        PaletteView {
            query: String::new(),
            rows: palette::filter(&actions(), ""),
            selected: 0,
        }
    }

    #[test]
    fn the_selection_walks_the_rows() {
        let mut state = view();

        state.move_selection(1);
        assert_eq!(state.selected, 1);

        state.move_selection(-1);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn the_selection_stops_at_the_ends() {
        let mut state = view();

        state.move_selection(-5);
        assert_eq!(state.selected, 0);

        state.move_selection(99);
        assert_eq!(state.selected, state.rows.len() - 1);
    }

    #[test]
    fn an_empty_list_has_nothing_to_run() {
        let mut state = PaletteView::default();
        state.move_selection(1);

        assert_eq!(state.selected, 0);
        assert_eq!(state.selected_id(), None);
    }

    #[test]
    fn the_selected_row_is_what_runs() {
        let mut state = view();
        state.move_selection(2);

        assert_eq!(state.selected_id(), Some(state.rows[2].id));
    }
}

mod geometry {
    use super::*;

    fn window() -> Rect {
        Rect::from_size(1440.0, 900.0)
    }

    #[test]
    fn the_panel_is_centred_below_the_top_edge() {
        let panel = palette::frame(window(), 5, 1.0);

        let left = panel.x;
        let right = window().right() - panel.right();
        assert!((left - right).abs() < 0.5, "off centre");
        assert!(panel.y > 0.0);
    }

    #[test]
    fn more_rows_make_a_taller_panel() {
        let few = palette::frame(window(), 2, 1.0);
        let many = palette::frame(window(), 8, 1.0);

        assert!(many.height > few.height);
    }

    #[test]
    fn the_panel_never_outgrows_the_window() {
        let small = Rect::from_size(420.0, 320.0);
        let panel = palette::frame(small, 8, 1.0);

        assert!(panel.right() <= small.right());
        assert!(panel.bottom() <= small.bottom());
    }

    #[test]
    fn the_panel_scales_with_the_display() {
        let one = palette::frame(window(), 5, 1.0);
        let two = palette::frame(window(), 5, 2.0);

        assert!(two.width > one.width);
        assert!(two.height > one.height);
    }

    #[test]
    fn rows_stack_under_the_input_without_overlapping() {
        let panel = palette::frame(window(), 4, 1.0);
        let input = palette::input_rect(panel, 1.0);

        let first = palette::row_rect(panel, 0, 1.0);
        assert!(first.y >= input.bottom());

        for index in 1..4 {
            let above = palette::row_rect(panel, index - 1, 1.0);
            let row = palette::row_rect(panel, index, 1.0);
            assert_eq!(above.bottom(), row.y);
        }
    }

    #[test]
    fn the_footer_sits_at_the_bottom() {
        let panel = palette::frame(window(), 3, 1.0);
        let footer = palette::footer_rect(panel, 1.0);

        assert_eq!(footer.bottom(), panel.bottom());
        assert!(footer.y > palette::row_rect(panel, 2, 1.0).y);
    }

    #[test]
    fn a_click_finds_the_row_under_it() {
        let panel = palette::frame(window(), 4, 1.0);
        let second = palette::row_rect(panel, 1, 1.0);

        assert_eq!(
            palette::row_at(panel, 4, 1.0, second.x + 20.0, second.y + 6.0),
            Some(1)
        );
    }

    #[test]
    fn a_click_outside_the_rows_finds_nothing() {
        let panel = palette::frame(window(), 4, 1.0);
        let input = palette::input_rect(panel, 1.0);

        assert_eq!(
            palette::row_at(panel, 4, 1.0, input.x + 20.0, input.y + 6.0),
            None
        );
        assert_eq!(palette::row_at(panel, 4, 1.0, 10.0, 800.0), None);
    }
}
