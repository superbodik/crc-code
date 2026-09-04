use crc_ui::geometry::Rect;
use crc_ui::view::menu::{self, MenuAction, MenuItem, MenuView};
use crc_ui::view::prompt::{self, PromptKind, PromptView, Target};

fn window() -> Rect {
    Rect::new(0.0, 0.0, 1440.0, 900.0)
}

mod what_the_menu_offers {
    use super::*;

    fn actions(view: &MenuView) -> Vec<MenuAction> {
        view.items
            .iter()
            .filter_map(|item| match item {
                MenuItem::Action(action) => Some(*action),
                MenuItem::Separator => None,
            })
            .collect()
    }

    #[test]
    fn a_file_can_be_made_renamed_deleted_and_found() {
        let view = MenuView::for_row("src/main.rs", false);
        let offered = actions(&view);

        assert!(offered.contains(&MenuAction::NewFile));
        assert!(offered.contains(&MenuAction::NewFolder));
        assert!(offered.contains(&MenuAction::Rename));
        assert!(offered.contains(&MenuAction::Delete));
        assert!(offered.contains(&MenuAction::CopyPath));
        assert!(offered.contains(&MenuAction::Reveal));
    }

    #[test]
    fn only_a_folder_is_worth_refreshing() {
        assert!(!actions(&MenuView::for_row("a.rs", false)).contains(&MenuAction::Refresh));
        assert!(actions(&MenuView::for_row("src", true)).contains(&MenuAction::Refresh));
    }

    #[test]
    fn empty_space_offers_only_what_makes_sense_there() {
        let offered = actions(&MenuView::for_root());

        assert_eq!(
            offered,
            vec![MenuAction::NewFile, MenuAction::NewFolder, MenuAction::Refresh]
        );
        assert!(
            !offered.contains(&MenuAction::Rename),
            "there is nothing there to rename"
        );
        assert!(!offered.contains(&MenuAction::Delete));
    }

    #[test]
    fn the_menu_remembers_what_it_was_raised_on() {
        assert_eq!(
            MenuView::for_row("src/main.rs", false).subject.as_deref(),
            Some("src/main.rs")
        );
        assert_eq!(MenuView::for_root().subject, None);
    }

    #[test]
    fn only_deleting_is_marked_as_destructive() {
        for action in [
            MenuAction::NewFile,
            MenuAction::NewFolder,
            MenuAction::Rename,
            MenuAction::CopyPath,
            MenuAction::Reveal,
            MenuAction::Refresh,
        ] {
            assert!(!action.destructive(), "{action:?} is not destructive");
        }
        assert!(MenuAction::Delete.destructive());
    }
}

mod where_it_lands {
    use super::*;

    #[test]
    fn the_menu_opens_at_the_pointer() {
        let view = MenuView::for_root().at(400.0, 300.0);
        let placed = menu::layout(window(), &view, 1.0);

        assert_eq!(placed.panel.x, 400.0);
        assert_eq!(placed.panel.y, 300.0);
    }

    #[test]
    fn a_menu_near_the_right_edge_is_pulled_back_inside() {
        let view = MenuView::for_root().at(1430.0, 300.0);
        let placed = menu::layout(window(), &view, 1.0);

        assert!(placed.panel.right() <= window().right());
        assert!(placed.panel.x >= window().x);
    }

    #[test]
    fn a_menu_near_the_bottom_is_pulled_up() {
        let view = MenuView::for_row("a.rs", true).at(400.0, 890.0);
        let placed = menu::layout(window(), &view, 1.0);

        assert!(placed.panel.bottom() <= window().bottom());
        assert!(placed.panel.y >= window().y);
    }

    #[test]
    fn every_item_gets_a_row_and_they_stack() {
        let view = MenuView::for_row("a.rs", true).at(100.0, 100.0);
        let placed = menu::layout(window(), &view, 1.0);

        assert_eq!(placed.rows.len(), view.items.len());
        for index in 1..placed.rows.len() {
            assert_eq!(placed.rows[index - 1].bottom(), placed.rows[index].y);
        }
        assert!(placed.rows.last().unwrap().bottom() <= placed.panel.bottom());
    }

    #[test]
    fn a_separator_is_shorter_than_an_action() {
        let view = MenuView::for_row("a.rs", false).at(100.0, 100.0);
        let placed = menu::layout(window(), &view, 1.0);

        let separator = view
            .items
            .iter()
            .position(|item| matches!(item, MenuItem::Separator))
            .unwrap();

        assert!(placed.rows[separator].height < placed.rows[0].height);
    }
}

mod pressing {
    use super::*;

    #[test]
    fn a_click_on_an_action_names_it() {
        let view = MenuView::for_root().at(200.0, 200.0);
        let placed = menu::layout(window(), &view, 1.0);

        for (index, item) in view.items.iter().enumerate() {
            let rect = placed.rows[index];
            let hit = menu::item_at(&placed, &view, rect.x + 20.0, rect.y + 4.0);

            match item {
                MenuItem::Action(_) => assert_eq!(hit, Some(index)),
                MenuItem::Separator => assert_eq!(hit, None, "a separator is not a button"),
            }
        }
    }

    #[test]
    fn a_click_outside_the_menu_names_nothing() {
        let view = MenuView::for_root().at(200.0, 200.0);
        let placed = menu::layout(window(), &view, 1.0);

        assert_eq!(menu::item_at(&placed, &view, 20.0, 20.0), None);
        assert_eq!(
            menu::item_at(&placed, &view, placed.panel.right() + 10.0, 210.0),
            None
        );
    }
}

mod asking_for_a_name {
    use super::*;

    #[test]
    fn a_name_must_be_something_a_file_system_will_take() {
        let mut prompt = PromptView::new(PromptKind::NewFile, "в корне");
        assert!(!prompt.ready(), "an empty name is not a name");

        prompt.value = "notes.txt".to_string();
        assert!(prompt.ready());

        for bad in ["a/b", "a\\b", "a:b", "a*b", "a?b", "..", "."] {
            prompt.value = bad.to_string();
            assert!(!prompt.ready(), "{bad} should be refused");
        }
    }

    #[test]
    fn surrounding_spaces_do_not_count_as_a_name() {
        let mut prompt = PromptView::new(PromptKind::NewFolder, "в корне");
        prompt.value = "   ".to_string();
        assert!(!prompt.ready());

        prompt.value = "  notes  ".to_string();
        assert!(prompt.ready());
        assert_eq!(prompt.trimmed(), "notes");
    }

    #[test]
    fn deleting_needs_no_name_at_all() {
        let prompt = PromptView::new(PromptKind::Delete, "a.rs уйдёт навсегда");

        assert!(prompt.ready(), "deleting asks for confirmation, not a name");
        assert!(!prompt.kind.asks_for_a_name());
        assert!(prompt.kind.destructive());
    }

    #[test]
    fn renaming_starts_from_the_current_name() {
        let prompt = PromptView::seeded(PromptKind::Rename, "src/main.rs", "main.rs");

        assert_eq!(prompt.value, "main.rs");
        assert!(prompt.ready());
    }

    #[test]
    fn each_kind_says_what_its_button_does() {
        assert_eq!(PromptKind::NewFile.confirm(), "Создать");
        assert_eq!(PromptKind::NewFolder.confirm(), "Создать");
        assert_eq!(PromptKind::Rename.confirm(), "Переименовать");
        assert_eq!(PromptKind::Delete.confirm(), "Удалить");
    }

    #[test]
    fn the_panel_is_centred_and_its_buttons_sit_inside_it() {
        let prompt = PromptView::new(PromptKind::NewFile, "в корне");
        let placed = prompt::layout(window(), &prompt, 1.0);

        let left = placed.panel.x - window().x;
        let right = window().right() - placed.panel.right();
        assert!((left - right).abs() < 0.5, "off centre");

        for rect in [placed.title, placed.field, placed.confirm, placed.cancel] {
            assert!(rect.x >= placed.panel.x);
            assert!(rect.right() <= placed.panel.right());
            assert!(rect.bottom() <= placed.panel.bottom() + 0.01);
        }
        assert!(placed.cancel.right() <= placed.confirm.x);
    }

    #[test]
    fn a_confirmation_needs_no_field_so_it_is_shorter() {
        let asking = prompt::layout(
            window(),
            &PromptView::new(PromptKind::NewFile, "в корне"),
            1.0,
        );
        let confirming = prompt::layout(
            window(),
            &PromptView::new(PromptKind::Delete, "уйдёт навсегда"),
            1.0,
        );

        assert!(confirming.panel.height < asking.panel.height);
        assert_eq!(confirming.field.height, 0.0);
    }

    #[test]
    fn the_buttons_answer_to_a_click_and_the_backdrop_does_not() {
        let prompt = PromptView::new(PromptKind::NewFile, "в корне");
        let placed = prompt::layout(window(), &prompt, 1.0);

        assert_eq!(
            prompt::target_at(&placed, placed.confirm.x + 4.0, placed.confirm.y + 4.0),
            Some(Target::Confirm)
        );
        assert_eq!(
            prompt::target_at(&placed, placed.cancel.x + 4.0, placed.cancel.y + 4.0),
            Some(Target::Cancel)
        );
        assert_eq!(
            prompt::target_at(&placed, placed.field.x + 10.0, placed.field.y + 4.0),
            Some(Target::Field)
        );
        assert_eq!(prompt::target_at(&placed, 10.0, 10.0), None);
    }

    #[test]
    fn a_confirmation_has_no_field_to_click_on() {
        let prompt = PromptView::new(PromptKind::Delete, "уйдёт навсегда");
        let placed = prompt::layout(window(), &prompt, 1.0);

        assert_eq!(
            prompt::target_at(&placed, placed.panel.x + 30.0, placed.note.y + 4.0),
            None
        );
    }
}
