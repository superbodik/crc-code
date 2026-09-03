use crc_theme::Theme;
use crc_ui::geometry::Rect;
use crc_ui::view::hit::{self, ExplorerButton};
use crc_ui::view::rail::{self, RailAction};

fn metrics() -> crc_theme::Metrics {
    Theme::dark().metrics()
}

fn rail_bar() -> Rect {
    Rect::new(0.0, 40.0, 44.0, 860.0)
}

fn sidebar() -> Rect {
    Rect::new(44.0, 40.0, 260.0, 860.0)
}

mod the_rail {
    use super::*;

    #[test]
    fn every_action_gets_a_button_and_they_stack_downwards() {
        let bar = rail_bar();
        let metrics = metrics();

        let mut previous: Option<Rect> = None;
        for index in 0..RailAction::ALL.len() {
            let button = rail::button(bar, &metrics, index);
            assert!(bar.contains(button.x, button.y), "button {index} left the rail");
            assert!(button.right() <= bar.right());
            if let Some(above) = previous {
                assert!(above.bottom() <= button.y, "buttons {index} overlap");
            }
            previous = Some(button);
        }
    }

    #[test]
    fn a_click_names_the_action_under_it() {
        let bar = rail_bar();
        let metrics = metrics();

        for (index, action) in RailAction::ALL.into_iter().enumerate() {
            let button = rail::button(bar, &metrics, index);
            assert_eq!(
                rail::action_at(bar, &metrics, button.x + 4.0, button.y + 4.0),
                Some(action)
            );
        }
    }

    #[test]
    fn the_gaps_and_the_space_below_press_nothing() {
        let bar = rail_bar();
        let metrics = metrics();
        let first = rail::button(bar, &metrics, 0);

        assert_eq!(rail::action_at(bar, &metrics, bar.x + 2.0, bar.y + 2.0), None);
        assert_eq!(
            rail::action_at(bar, &metrics, first.x + 4.0, bar.bottom() - 20.0),
            None
        );
    }

    #[test]
    fn nothing_outside_the_rail_is_a_rail_button() {
        let bar = rail_bar();
        let metrics = metrics();
        let first = rail::button(bar, &metrics, 0);

        assert_eq!(
            rail::action_at(bar, &metrics, bar.right() + 10.0, first.y + 4.0),
            None
        );
    }

    #[test]
    fn every_action_carries_a_glyph_and_a_name() {
        for action in RailAction::ALL {
            assert!(!action.title().is_empty());
            assert_ne!(action.glyph(), '\0');
        }
    }
}

mod the_explorer_header {
    use super::*;

    #[test]
    fn both_buttons_sit_in_the_header_at_its_right_edge() {
        let bar = sidebar();
        let metrics = metrics();
        let header = hit::explorer_header_height(&metrics);

        for index in 0..ExplorerButton::ALL.len() {
            let button = hit::explorer_button(bar, &metrics, index);
            assert!(button.bottom() <= bar.y + header, "button {index} spills out");
            assert!(button.right() <= bar.right());
            assert!(button.x > bar.x + bar.width / 2.0, "buttons belong on the right");
        }
    }

    #[test]
    fn the_buttons_do_not_overlap_each_other() {
        let bar = sidebar();
        let metrics = metrics();
        let first = hit::explorer_button(bar, &metrics, 0);
        let second = hit::explorer_button(bar, &metrics, 1);

        assert!(second.right() <= first.x, "the two buttons sit on top of each other");
    }

    #[test]
    fn a_click_names_the_button_under_it() {
        let bar = sidebar();
        let metrics = metrics();

        for (index, button) in ExplorerButton::ALL.into_iter().enumerate() {
            let rect = hit::explorer_button(bar, &metrics, index);
            assert_eq!(
                hit::explorer_button_at(bar, &metrics, rect.x + 4.0, rect.y + 4.0),
                Some(button)
            );
        }
    }

    #[test]
    fn the_title_is_not_a_button_and_neither_is_the_tree() {
        let bar = sidebar();
        let metrics = metrics();
        let header = hit::explorer_header_height(&metrics);

        assert_eq!(
            hit::explorer_button_at(bar, &metrics, bar.x + 20.0, bar.y + header / 2.0),
            None
        );
        assert_eq!(
            hit::explorer_button_at(bar, &metrics, bar.right() - 20.0, bar.y + header + 40.0),
            None,
            "a row of the tree must never fire the header buttons"
        );
    }

    #[test]
    fn a_row_of_the_tree_is_still_reachable_under_the_buttons() {
        let bar = sidebar();
        let metrics = metrics();
        let header = hit::explorer_header_height(&metrics);

        assert_eq!(
            hit::explorer_row(bar, &metrics, bar.y + header + metrics.row_height * 2.5),
            Some(2)
        );
    }
}
