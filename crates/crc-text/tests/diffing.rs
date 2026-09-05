use crc_text::diff::{Line, around, lines, tally};

fn shape(changes: &[Line]) -> String {
    changes.iter().map(Line::marker).collect()
}

fn texts(changes: &[Line]) -> Vec<&str> {
    changes.iter().map(Line::text).collect()
}

mod comparing {
    use super::*;

    #[test]
    fn two_files_that_match_have_nothing_but_common_lines() {
        let same = "one\ntwo\nthree";
        let changes = lines(same, same);

        assert_eq!(shape(&changes), "   ");
        assert_eq!(tally(&changes), (0, 0));
    }

    #[test]
    fn a_line_changed_in_the_middle_is_one_out_and_one_in() {
        let changes = lines("one\ntwo\nthree", "one\nTWO\nthree");

        assert_eq!(shape(&changes), " -+ ");
        assert_eq!(tally(&changes), (1, 1));
        assert_eq!(texts(&changes), vec!["one", "two", "TWO", "three"]);
    }

    #[test]
    fn a_line_added_is_only_an_addition() {
        let changes = lines("one\ntwo", "one\nmiddle\ntwo");

        assert_eq!(shape(&changes), " + ");
        assert_eq!(tally(&changes), (1, 0));
    }

    #[test]
    fn a_line_removed_is_only_a_removal() {
        let changes = lines("one\ngone\ntwo", "one\ntwo");

        assert_eq!(shape(&changes), " - ");
        assert_eq!(tally(&changes), (0, 1));
    }

    #[test]
    fn a_new_file_is_all_additions() {
        let changes = lines("", "one\ntwo");

        assert_eq!(shape(&changes), "++");
        assert_eq!(tally(&changes), (2, 0));
    }

    #[test]
    fn an_emptied_file_is_all_removals() {
        let changes = lines("one\ntwo", "");

        assert_eq!(shape(&changes), "--");
        assert_eq!(tally(&changes), (0, 2));
    }

    #[test]
    fn a_move_reads_as_a_removal_and_an_addition_not_as_a_rewrite() {
        let changes = lines("a\nb\nc", "b\nc\na");

        let (added, removed) = tally(&changes);
        assert_eq!(added, 1);
        assert_eq!(removed, 1);
        assert_eq!(
            changes.iter().filter(|line| line.marker() == ' ').count(),
            2,
            "b and c should be recognised as unchanged"
        );
    }

    #[test]
    fn every_kept_line_knows_where_it_sits_on_both_sides() {
        let changes = lines("one\ntwo\nthree", "one\nTWO\nthree");

        match &changes[0] {
            Line::Same { before, after, .. } => {
                assert_eq!((*before, *after), (0, 0));
            }
            other => panic!("expected a kept line, got {other:?}"),
        }
        match &changes[3] {
            Line::Same { before, after, .. } => {
                assert_eq!((*before, *after), (2, 2), "the tail keeps its numbers");
            }
            other => panic!("expected a kept line, got {other:?}"),
        }
    }

    #[test]
    fn a_change_at_the_very_start_is_found() {
        let changes = lines("one\ntwo", "ONE\ntwo");
        assert_eq!(shape(&changes), "-+ ");
    }

    #[test]
    fn a_change_at_the_very_end_is_found() {
        let changes = lines("one\ntwo", "one\nTWO");
        assert_eq!(shape(&changes), " -+");
    }

    #[test]
    fn a_huge_rewrite_does_not_try_to_align_line_by_line() {
        let before = (0..2000)
            .map(|index| format!("old {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let after = (0..2000)
            .map(|index| format!("new {index}"))
            .collect::<Vec<_>>()
            .join("\n");

        let changes = lines(&before, &after);
        let (added, removed) = tally(&changes);

        assert_eq!(added, 2000);
        assert_eq!(removed, 2000);
    }
}

mod trimming {
    use super::*;

    #[test]
    fn only_the_neighbourhood_of_a_change_is_kept() {
        let before = (0..40)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut after: Vec<String> = before.lines().map(str::to_string).collect();
        after[20] = "changed".to_string();

        let changes = around(lines(&before, &after.join("\n")), 2);

        assert!(changes.len() < 12, "far too much context: {}", changes.len());
        assert!(
            changes.iter().any(|line| matches!(line, Line::Skipped(_))),
            "the skipped stretch should be marked"
        );
    }

    #[test]
    fn a_file_with_no_changes_shows_nothing_at_all() {
        let same = "one\ntwo\nthree";
        assert!(around(lines(same, same), 3).is_empty());
    }

    #[test]
    fn the_count_of_skipped_lines_is_reported() {
        let before = (0..30)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut after: Vec<String> = before.lines().map(str::to_string).collect();
        after[0] = "changed".to_string();

        let changes = around(lines(&before, &after.join("\n")), 1);

        match changes.last() {
            Some(Line::Skipped(count)) => assert!(*count > 20, "expected a big gap, got {count}"),
            other => panic!("expected a gap at the end, got {other:?}"),
        }
    }

    #[test]
    fn two_changes_far_apart_keep_their_own_context() {
        let before = (0..40)
            .map(|index| format!("line {index}"))
            .collect::<Vec<_>>()
            .join("\n");
        let mut after: Vec<String> = before.lines().map(str::to_string).collect();
        after[5] = "first".to_string();
        after[35] = "second".to_string();

        let changes = around(lines(&before, &after.join("\n")), 2);
        let gaps = changes
            .iter()
            .filter(|line| matches!(line, Line::Skipped(_)))
            .count();

        assert_eq!(gaps, 3, "before, between and after");
    }
}
