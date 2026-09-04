use crc_theme::Theme;
use crc_ui::geometry::Rect;
use crc_ui::view::panel::{self, PanelTab, PanelView, Problem, Target};

fn panel() -> Rect {
    Rect::new(300.0, 600.0, 1000.0, 220.0)
}

fn metrics() -> crc_theme::Metrics {
    Theme::dark().metrics()
}

fn placed(view: &PanelView) -> panel::Layout {
    panel::layout(panel(), view, &metrics(), 7.0)
}

fn problem(line: usize, message: &str) -> Problem {
    Problem {
        file: "broken.rs".to_string(),
        line,
        column: 4,
        message: message.to_string(),
    }
}

fn view() -> PanelView {
    PanelView {
        tab: PanelTab::Problems,
        problems: vec![
            problem(2, "не хватает )"),
            problem(3, "не хватает ;"),
            problem(9, "разбор здесь ломается"),
        ],
        output: vec!["видеокарта: NVIDIA".to_string(), "проект: bench".to_string()],
        ..PanelView::default()
    }
}

mod counting {
    use super::*;

    #[test]
    fn each_tab_counts_its_own_rows() {
        let state = view();

        assert_eq!(state.count(PanelTab::Problems), 3);
        assert_eq!(state.count(PanelTab::Output), 2);
    }

    #[test]
    fn the_row_count_follows_the_tab_on_show() {
        let mut state = view();
        assert_eq!(state.rows(), 3);

        state.tab = PanelTab::Output;
        assert_eq!(state.rows(), 2);
    }

    #[test]
    fn an_empty_tab_explains_itself_in_its_own_words() {
        let mut state = PanelView::default();
        assert_eq!(state.empty_note(), "Оболочка не запущена");

        state.tab = PanelTab::Problems;
        assert_eq!(state.empty_note(), "Разбор проходит без ошибок");

        state.tab = PanelTab::Output;
        assert_eq!(state.empty_note(), "Пока нечего показать");
    }
}

mod placing {
    use super::*;

    #[test]
    fn every_tab_gets_a_rectangle_and_they_run_left_to_right() {
        let layout = placed(&view());

        assert_eq!(layout.tabs.len(), PanelTab::ALL.len());
        for index in 1..layout.tabs.len() {
            assert!(
                layout.tabs[index - 1].right() <= layout.tabs[index].x,
                "tabs {index} overlap"
            );
        }
        assert!(layout.tabs.last().unwrap().right() < panel().right());
    }

    #[test]
    fn a_tab_with_a_count_is_wider_than_one_without() {
        let empty = placed(&PanelView::default());
        let full = placed(&view());

        let problems = PanelTab::ALL
            .iter()
            .position(|tab| *tab == PanelTab::Problems)
            .unwrap();

        assert!(
            full.tabs[problems].width > empty.tabs[problems].width,
            "the badge should make room for itself"
        );
        assert_eq!(
            full.tabs[0].width, empty.tabs[0].width,
            "the terminal never wears a badge, so its width never moves"
        );
    }

    #[test]
    fn the_rows_sit_below_the_header_and_inside_the_panel() {
        let layout = placed(&view());

        assert!(layout.header.bottom() <= layout.body.y);
        assert_eq!(layout.rows.len(), 3);
        for row in &layout.rows {
            assert!(row.y >= layout.body.y);
            assert!(row.bottom() <= layout.body.bottom() + 0.01);
        }
    }

    #[test]
    fn a_long_list_is_clipped_to_the_panel() {
        let mut state = view();
        state.problems = (0..500).map(|line| problem(line, "плохо")).collect();

        let layout = placed(&state);
        assert!(layout.rows.len() < 500);
        assert!(layout.rows.last().unwrap().bottom() <= layout.body.bottom() + 0.01);
    }

    #[test]
    fn a_shallow_panel_still_lays_out() {
        let shallow = Rect::new(300.0, 600.0, 1000.0, 30.0);
        let state = view();
        let layout = panel::layout(shallow, &state, &metrics(), 7.0);

        assert!(layout.body.height >= 0.0);
        assert!(layout.rows.iter().all(|row| row.height > 0.0));
    }
}

mod clicking {
    use super::*;

    #[test]
    fn a_click_on_a_tab_names_it() {
        let state = view();
        let layout = placed(&state);

        for index in 0..PanelTab::ALL.len() {
            let tab = layout.tabs[index];
            assert_eq!(
                panel::target_at(&layout, &state, tab.x + 4.0, tab.y + 4.0),
                Some(Target::Tab(index))
            );
        }
    }

    #[test]
    fn a_click_on_a_row_names_its_index() {
        let state = view();
        let layout = placed(&state);

        for index in 0..state.rows() {
            let row = layout.rows[index];
            assert_eq!(
                panel::target_at(&layout, &state, row.x + 40.0, row.y + 2.0),
                Some(Target::Row(index))
            );
        }
    }

    #[test]
    fn scrolling_shifts_which_row_a_click_lands_on() {
        let mut state = view();
        state.scroll = 2;
        let layout = placed(&state);

        assert_eq!(
            panel::target_at(&layout, &state, layout.rows[0].x + 40.0, layout.rows[0].y + 2.0),
            Some(Target::Row(2))
        );
    }

    #[test]
    fn empty_space_below_the_rows_names_nothing() {
        let state = view();
        let layout = placed(&state);
        let below = layout.rows.last().unwrap().bottom() + 30.0;

        assert_eq!(
            panel::target_at(&layout, &state, layout.body.x + 40.0, below),
            None
        );
    }

    #[test]
    fn the_gap_between_the_tabs_and_the_header_edge_is_not_a_tab() {
        let state = view();
        let layout = placed(&state);

        assert_eq!(
            panel::target_at(&layout, &state, layout.header.right() - 20.0, layout.header.y + 8.0),
            None
        );
    }
}

mod the_shell_tab {
    use super::*;

    #[test]
    fn the_terminal_leads_the_strip() {
        assert_eq!(PanelTab::ALL[0], PanelTab::Terminal);
        assert_eq!(PanelTab::default(), PanelTab::Terminal);
    }

    #[test]
    fn only_the_terminal_tab_shows_a_shell() {
        let mut state = view();

        state.tab = PanelTab::Terminal;
        assert!(state.shows_a_shell());

        state.tab = PanelTab::Problems;
        assert!(!state.shows_a_shell());

        state.tab = PanelTab::Output;
        assert!(!state.shows_a_shell());
    }

    #[test]
    fn the_terminal_carries_no_row_count_of_its_own() {
        let mut state = view();
        state.tab = PanelTab::Terminal;

        assert_eq!(state.rows(), 0, "the shell is not a list of rows");
        assert_eq!(state.count(PanelTab::Terminal), 0, "and wears no badge");
    }

    #[test]
    fn a_shell_that_never_started_says_so() {
        let mut state = view();
        state.tab = PanelTab::Terminal;

        assert_eq!(state.empty_note(), "Оболочка не запущена");
    }

    #[test]
    fn a_click_on_a_tab_is_never_a_click_into_the_shell() {
        let mut state = view();
        state.tab = PanelTab::Terminal;
        let layout = placed(&state);

        let tab = layout.tabs[1];
        assert_eq!(
            panel::target_at(&layout, &state, tab.x + 4.0, tab.y + 4.0),
            Some(Target::Tab(1)),
            "the strip stays reachable while a shell is running"
        );
    }
}
