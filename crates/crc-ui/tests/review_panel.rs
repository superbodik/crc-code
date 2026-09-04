use crc_ui::geometry::Rect;
use crc_ui::view::review::{self, ReviewView, Target, describe};
use serde_json::json;

fn window() -> Rect {
    Rect::new(0.0, 0.0, 1440.0, 900.0)
}

fn view(tool: &str, file: Option<&str>, detail: Vec<&str>) -> ReviewView {
    ReviewView {
        tool: tool.to_string(),
        file: file.map(str::to_string),
        detail: detail.into_iter().map(str::to_string).collect(),
        hovered: None,
    }
}

mod naming_what_is_asked {
    use super::*;

    #[test]
    fn each_tool_is_described_in_words_not_in_its_own_name() {
        assert_eq!(view("Write", None, vec![]).title(), "Записать файл");
        assert_eq!(view("Edit", None, vec![]).title(), "Изменить файл");
        assert_eq!(view("Bash", None, vec![]).title(), "Выполнить команду");
    }

    #[test]
    fn a_tool_with_no_wording_of_its_own_still_gets_a_title() {
        assert_eq!(view("Whatever", None, vec![]).title(), "Разрешить Whatever");
    }

    #[test]
    fn the_subject_is_the_file_when_there_is_one() {
        assert_eq!(view("Edit", Some("src/main.rs"), vec![]).subject(), "src/main.rs");
        assert_eq!(view("Bash", None, vec![]).subject(), "Bash");
    }
}

mod showing_the_detail {
    use super::*;

    #[test]
    fn a_command_is_shown_under_its_own_heading() {
        let lines = describe(&json!({ "command": "cargo test" }));

        assert_eq!(lines[0], "command:");
        assert_eq!(lines[1], "  cargo test");
    }

    #[test]
    fn an_edit_shows_both_sides_of_the_change() {
        let lines = describe(&json!({
            "file_path": "a.rs",
            "old_string": "let x = 1;",
            "new_string": "let x = 2;"
        }));

        assert!(lines.contains(&"old_string:".to_string()));
        assert!(lines.contains(&"new_string:".to_string()));
        assert!(lines.contains(&"  let x = 1;".to_string()));
        assert!(lines.contains(&"  let x = 2;".to_string()));
    }

    #[test]
    fn a_long_body_is_cut_off_rather_than_shown_whole() {
        let body = (0..80)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let lines = describe(&json!({ "content": body }));

        assert!(lines.len() <= 40, "the panel must stay readable");
        assert!(lines.iter().any(|line| line.trim() == "..."));
    }

    #[test]
    fn a_very_wide_line_is_trimmed() {
        let lines = describe(&json!({ "command": "x".repeat(400) }));

        for line in &lines {
            assert!(line.chars().count() <= 95, "too wide: {}", line.len());
        }
    }

    #[test]
    fn fields_nobody_needs_to_read_are_left_out() {
        let lines = describe(&json!({ "file_path": "a.rs", "limit": 200, "offset": 0 }));
        assert!(lines.is_empty(), "a path alone is already in the subject");
    }

    #[test]
    fn input_that_is_not_an_object_describes_nothing() {
        assert!(describe(&json!("just a string")).is_empty());
        assert!(describe(&json!(null)).is_empty());
    }
}

mod placing {
    use super::*;

    #[test]
    fn the_panel_is_centred_and_everything_sits_inside_it() {
        let state = view("Edit", Some("a.rs"), vec!["old_string:", "  let x = 1;"]);
        let placed = review::layout(window(), &state, 1.0);

        let left = placed.panel.x - window().x;
        let right = window().right() - placed.panel.right();
        assert!((left - right).abs() < 0.5, "off centre");

        for rect in [placed.title, placed.subject, placed.body, placed.allow, placed.deny] {
            assert!(rect.x >= placed.panel.x);
            assert!(rect.right() <= placed.panel.right());
            assert!(rect.bottom() <= placed.panel.bottom() + 0.01);
        }
    }

    #[test]
    fn the_buttons_do_not_overlap_and_allow_sits_on_the_right() {
        let state = view("Edit", Some("a.rs"), vec![]);
        let placed = review::layout(window(), &state, 1.0);

        assert!(placed.deny.right() <= placed.allow.x);
        assert!(placed.allow.right() <= placed.panel.right());
    }

    #[test]
    fn a_longer_detail_makes_a_taller_panel() {
        let short = review::layout(window(), &view("Edit", None, vec!["a"]), 1.0);
        let long = review::layout(
            window(),
            &view("Edit", None, (0..15).map(|_| "a line").collect()),
            1.0,
        );

        assert!(long.panel.height > short.panel.height);
        assert!(long.rows.len() > short.rows.len());
    }

    #[test]
    fn a_flood_of_detail_never_pushes_the_panel_off_the_screen() {
        let many: Vec<&str> = (0..400).map(|_| "a line").collect();
        let placed = review::layout(window(), &view("Write", None, many), 1.0);

        assert!(placed.panel.bottom() <= window().bottom());
        assert!(placed.panel.height <= window().height);
        assert!(placed.rows.last().unwrap().bottom() <= placed.body.bottom() + 0.01);
    }
}

mod deciding {
    use super::*;

    #[test]
    fn both_buttons_answer_to_a_click() {
        let state = view("Edit", Some("a.rs"), vec![]);
        let placed = review::layout(window(), &state, 1.0);

        assert_eq!(
            review::target_at(&placed, placed.allow.x + 4.0, placed.allow.y + 4.0),
            Some(Target::Allow)
        );
        assert_eq!(
            review::target_at(&placed, placed.deny.x + 4.0, placed.deny.y + 4.0),
            Some(Target::Deny)
        );
    }

    #[test]
    fn the_body_and_the_backdrop_decide_nothing() {
        let state = view("Edit", Some("a.rs"), vec!["a line"]);
        let placed = review::layout(window(), &state, 1.0);

        assert_eq!(
            review::target_at(&placed, placed.body.x + 20.0, placed.body.y + 4.0),
            None
        );
        assert_eq!(review::target_at(&placed, 10.0, 10.0), None);
    }
}

mod naming_a_long_path {
    use super::*;
    use crc_ui::view::review::tail;

    #[test]
    fn a_short_path_is_shown_whole() {
        assert_eq!(tail("src/main.rs", 82), "src/main.rs");
    }

    #[test]
    fn a_long_path_keeps_its_end_because_that_is_the_file() {
        let long = format!("C:/{}/hello.txt", "folder/".repeat(30));
        let shown = tail(&long, 40);

        assert!(shown.chars().count() <= 40);
        assert!(shown.starts_with("..."), "the cut is marked");
        assert!(
            shown.ends_with("hello.txt"),
            "the name must survive: {shown}"
        );
    }

    #[test]
    fn the_subject_of_a_deep_file_still_names_it() {
        let deep = format!("C:/{}/deep.rs", "a/".repeat(60));
        let state = view("Write", Some(&deep), vec![]);

        assert!(state.subject().ends_with("deep.rs"));
    }
}
