use std::path::PathBuf;

use crc_theme::Theme;
use crc_ui::geometry::Rect;
use crc_ui::view::search::{self, SearchRow, SearchView, Target};

fn sidebar() -> Rect {
    Rect::new(44.0, 40.0, 280.0, 700.0)
}

fn metrics() -> crc_theme::Metrics {
    Theme::dark().metrics()
}

fn results() -> Vec<(PathBuf, Vec<(u64, String)>)> {
    vec![
        (
            PathBuf::from("crates/crc-ui/src/view/rail.rs"),
            vec![(31, "            RailAction::Search => \"Поиск\",".to_string())],
        ),
        (
            PathBuf::from("crates/crc-ui/src/view/shell.rs"),
            vec![
                (1121, "    \"Поиск по командам\",".to_string()),
                (1499, "    \"Поиск по проекту\",".to_string()),
            ],
        ),
    ]
}

fn view() -> SearchView {
    let (rows, files, hits) = SearchView::fold(&results());
    SearchView {
        query: "поиск".to_string(),
        rows,
        files,
        hits,
        searched: true,
        ..SearchView::default()
    }
}

mod folding {
    use super::*;

    #[test]
    fn a_file_row_is_followed_by_its_lines() {
        let (rows, files, hits) = SearchView::fold(&results());

        assert_eq!(files, 2);
        assert_eq!(hits, 3);
        assert_eq!(rows.len(), 5, "two headers and three lines");

        assert!(rows[0].is_file());
        assert!(!rows[1].is_file());
        assert!(rows[2].is_file());
        assert!(!rows[3].is_file());
        assert!(!rows[4].is_file());
    }

    #[test]
    fn a_file_row_carries_its_own_count() {
        let (rows, _, _) = SearchView::fold(&results());

        match &rows[2] {
            SearchRow::File { hits, .. } => assert_eq!(*hits, 2),
            _ => panic!("the third row should head a file"),
        }
    }

    #[test]
    fn every_row_knows_which_file_it_belongs_to() {
        let (rows, _, _) = SearchView::fold(&results());
        let shell = PathBuf::from("crates/crc-ui/src/view/shell.rs");

        assert_eq!(rows[2].path(), &shell);
        assert_eq!(rows[3].path(), &shell, "a line row carries its file too");
        assert_eq!(rows[4].path(), &shell);
    }

    #[test]
    fn trailing_whitespace_is_trimmed_off_the_preview() {
        let noisy = vec![(
            PathBuf::from("a.rs"),
            vec![(1, "let rect = 1;   \n".to_string())],
        )];
        let (rows, _, _) = SearchView::fold(&noisy);

        match &rows[1] {
            SearchRow::Line { text, .. } => assert_eq!(text, "let rect = 1;"),
            _ => panic!("expected a line"),
        }
    }

    #[test]
    fn no_results_fold_to_nothing() {
        let (rows, files, hits) = SearchView::fold(&[]);

        assert!(rows.is_empty());
        assert_eq!(files, 0);
        assert_eq!(hits, 0);
    }
}

mod telling_the_user {
    use super::*;

    #[test]
    fn an_empty_query_asks_for_one() {
        let state = SearchView::default();
        assert_eq!(state.tally(), "Введи, что искать");
    }

    #[test]
    fn a_query_not_yet_run_says_so() {
        let state = SearchView {
            query: "rect".to_string(),
            ..SearchView::default()
        };
        assert_eq!(state.tally(), "Ищу...");
    }

    #[test]
    fn a_finished_search_with_nothing_found_says_that() {
        let state = SearchView {
            query: "nowhere".to_string(),
            searched: true,
            ..SearchView::default()
        };
        assert_eq!(state.tally(), "Ничего не нашлось");
    }

    #[test]
    fn a_finished_search_counts_hits_and_files() {
        assert_eq!(view().tally(), "3 в 2 файлах");
    }
}

mod placing {
    use super::*;

    #[test]
    fn the_field_the_tally_and_the_list_stack_without_overlapping() {
        let placed = search::layout(sidebar(), &view(), &metrics());

        assert!(placed.header.bottom() <= placed.field.y);
        assert!(placed.field.bottom() <= placed.tally.y);
        assert!(placed.tally.bottom() <= placed.list.y);
        assert!(placed.list.bottom() <= sidebar().bottom());
    }

    #[test]
    fn the_case_button_sits_beside_the_field_not_on_it() {
        let placed = search::layout(sidebar(), &view(), &metrics());

        assert!(placed.field.right() <= placed.match_case.x);
        assert!(placed.match_case.right() <= sidebar().right());
    }

    #[test]
    fn every_row_that_fits_gets_a_rectangle() {
        let placed = search::layout(sidebar(), &view(), &metrics());

        assert_eq!(placed.rows.len(), view().rows.len());
        for row in &placed.rows {
            assert!(row.right() <= sidebar().right());
            assert!(row.bottom() <= placed.list.bottom() + 0.01);
        }
    }

    #[test]
    fn a_long_list_is_clipped_to_the_panel() {
        let mut state = view();
        state.rows = (0..400)
            .map(|index| SearchRow::Line {
                path: PathBuf::from("a.rs"),
                line: index,
                text: format!("line {index}"),
            })
            .collect();

        let placed = search::layout(sidebar(), &state, &metrics());
        assert!(placed.rows.len() < 400);
        assert!(placed.rows.last().unwrap().bottom() <= placed.list.bottom() + 0.01);
    }

    #[test]
    fn scrolling_shifts_which_row_a_click_lands_on() {
        let mut state = view();
        state.scroll = 2;
        let placed = search::layout(sidebar(), &state, &metrics());
        let row = placed.rows[0];

        assert_eq!(
            search::target_at(&placed, &state, row.x + 20.0, row.y + 4.0),
            Some(Target::Row(2))
        );
    }
}

mod clicking {
    use super::*;

    #[test]
    fn a_click_on_a_row_names_its_index() {
        let state = view();
        let placed = search::layout(sidebar(), &state, &metrics());

        for index in 0..state.rows.len() {
            let row = placed.rows[index];
            assert_eq!(
                search::target_at(&placed, &state, row.x + 20.0, row.y + 4.0),
                Some(Target::Row(index))
            );
        }
    }

    #[test]
    fn the_field_and_the_case_button_are_told_apart() {
        let state = view();
        let placed = search::layout(sidebar(), &state, &metrics());

        assert_eq!(
            search::target_at(&placed, &state, placed.field.x + 10.0, placed.field.y + 8.0),
            Some(Target::Field)
        );
        assert_eq!(
            search::target_at(
                &placed,
                &state,
                placed.match_case.x + 4.0,
                placed.match_case.y + 4.0
            ),
            Some(Target::MatchCase)
        );
    }

    #[test]
    fn empty_space_below_the_results_names_nothing() {
        let state = view();
        let placed = search::layout(sidebar(), &state, &metrics());
        let below = placed.rows.last().unwrap().bottom() + 40.0;

        assert_eq!(
            search::target_at(&placed, &state, placed.list.x + 20.0, below),
            None
        );
    }
}
