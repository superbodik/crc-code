use crc_theme::{Rgba, Theme};
use crc_ui::geometry::Rect;
use crc_ui::view::find::{self, FindView, Target};
use crc_ui::view::{self, CodeMetrics, EditorView};
use crc_ui::{Offscreen, Shell, ShellState};

fn buffer() -> Rect {
    Rect::new(300.0, 120.0, 1000.0, 600.0)
}

fn state(query: &str, total: usize) -> FindView {
    FindView {
        query: query.to_string(),
        total,
        current: 0,
        match_case: false,
        hovered: None,
    }
}

mod counting {
    use super::*;

    #[test]
    fn the_tally_reads_one_based() {
        let mut found = state("rect", 7);
        assert_eq!(found.tally(), "1 из 7");

        found.current = 3;
        assert_eq!(found.tally(), "4 из 7");
    }

    #[test]
    fn an_empty_query_says_nothing_at_all() {
        assert_eq!(state("", 0).tally(), "");
    }

    #[test]
    fn a_query_with_no_hits_says_so() {
        assert_eq!(state("nowhere", 0).tally(), "нет совпадений");
    }

    #[test]
    fn stepping_wraps_around_in_both_directions() {
        let mut found = state("rect", 3);

        found.step(true);
        assert_eq!(found.current, 1);
        found.step(true);
        found.step(true);
        assert_eq!(
            found.current, 0,
            "forward past the end comes back to the first"
        );

        found.step(false);
        assert_eq!(found.current, 2, "back past the start lands on the last");
    }

    #[test]
    fn stepping_with_no_matches_stays_put() {
        let mut found = state("nowhere", 0);
        found.step(true);
        found.step(false);
        assert_eq!(found.current, 0);
    }
}

mod placing {
    use super::*;

    #[test]
    fn the_bar_sits_at_the_top_right_of_the_buffer() {
        let placed = find::layout(buffer(), 1.0);

        assert!(placed.bar.right() < buffer().right());
        assert!(placed.bar.y > buffer().y);
        assert!(placed.bar.bottom() < buffer().bottom());
        assert!(
            placed.bar.x > buffer().x + buffer().width / 2.0,
            "it must not cover the left edge where the code starts"
        );
    }

    #[test]
    fn everything_sits_inside_the_bar_and_in_order() {
        let placed = find::layout(buffer(), 1.0);

        for (name, rect) in [
            ("field", placed.field),
            ("tally", placed.tally),
            ("case", placed.match_case),
            ("previous", placed.previous),
            ("next", placed.next),
            ("close", placed.close),
        ] {
            assert!(
                rect.x >= placed.bar.x && rect.right() <= placed.bar.right(),
                "{name} spills out of the bar"
            );
        }

        assert!(placed.field.right() <= placed.tally.x);
        assert!(placed.tally.right() <= placed.match_case.x);
        assert!(placed.match_case.right() <= placed.previous.x);
        assert!(placed.previous.right() <= placed.next.x);
        assert!(placed.next.right() <= placed.close.x);
    }

    #[test]
    fn a_narrow_buffer_squeezes_the_bar_instead_of_breaking_it() {
        let narrow = Rect::new(300.0, 120.0, 320.0, 600.0);
        let placed = find::layout(narrow, 1.0);

        assert!(placed.bar.width <= narrow.width);
        assert!(placed.bar.x >= narrow.x);
        assert!(placed.field.width >= 0.0);
    }

    #[test]
    fn it_scales_with_the_display() {
        let one = find::layout(buffer(), 1.0);
        let two = find::layout(buffer(), 2.0);

        assert!(two.bar.height > one.bar.height);
        assert!(two.close.width > one.close.width);
    }
}

mod clicking {
    use super::*;

    #[test]
    fn every_button_answers_to_a_click() {
        let placed = find::layout(buffer(), 1.0);

        for (target, rect) in [
            (Target::Close, placed.close),
            (Target::Next, placed.next),
            (Target::Previous, placed.previous),
            (Target::MatchCase, placed.match_case),
            (Target::Field, placed.field),
        ] {
            assert_eq!(
                find::target_at(&placed, rect.x + 2.0, rect.y + 2.0),
                Some(target)
            );
        }
    }

    #[test]
    fn the_code_around_the_bar_is_not_the_bar() {
        let placed = find::layout(buffer(), 1.0);

        assert_eq!(
            find::target_at(&placed, buffer().x + 20.0, buffer().y + 20.0),
            None
        );
        assert_eq!(
            find::target_at(&placed, placed.bar.x + 4.0, placed.bar.bottom() + 20.0),
            None
        );
    }

    #[test]
    fn the_tally_is_not_a_button() {
        let placed = find::layout(buffer(), 1.0);

        assert_eq!(
            find::target_at(&placed, placed.tally.x + 4.0, placed.tally.y + 20.0),
            None
        );
    }
}

mod drawing {
    use super::*;

    const WIDTH: u32 = 1100;
    const HEIGHT: u32 = 700;

    fn near(a: Rgba, b: Rgba) -> bool {
        a.r.abs_diff(b.r) <= 3 && a.g.abs_diff(b.g) <= 3 && a.b.abs_diff(b.b) <= 3
    }

    fn editor(matches: Vec<std::ops::Range<usize>>, current: Option<usize>) -> EditorView {
        EditorView {
            project: "crc-code".to_string(),
            focused: true,
            text: "let rect = Rect::new();\nlet other = rect.width;\nrect\n".to_string(),
            find: Some(state("rect", matches.len())),
            matches,
            current_match: current,
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
    fn the_bar_paints_over_the_code() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(Vec::new(), None),
            CodeMetrics::default(),
        ));

        let placed = find::layout(layout.buffer, theme.scale);
        assert!(
            canvas.count_pixels(&pixels, placed.bar, |c| near(c, theme.chrome.raised)) > 2000,
            "the find bar did not paint"
        );
    }

    #[test]
    fn matches_are_marked_and_the_current_one_stands_out() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(vec![4..8, 36..40, 48..52], Some(1)),
            CodeMetrics::default(),
        ));

        let solid = canvas.count_pixels(&pixels, layout.buffer, |c| {
            near(c, theme.chrome.accent_solid)
        });
        let wash = canvas.count_pixels(&pixels, layout.buffer, |c| {
            near(c, theme.chrome.accent_wash)
        });

        assert!(solid > 100, "the current match is not filled: {solid}");
        assert!(wash > 100, "the other matches are not marked: {wash}");
    }

    #[test]
    fn with_no_matches_the_code_is_left_alone() {
        let mut canvas = Offscreen::new(WIDTH, HEIGHT).expect("a GPU");
        let theme = Theme::dark();
        let layout = shell(&theme);

        let pixels = canvas.render_frame(&view::draw(
            &layout,
            &theme,
            &editor(Vec::new(), None),
            CodeMetrics::default(),
        ));

        let code = Rect::new(
            layout.buffer.x,
            layout.buffer.y + 120.0,
            layout.buffer.width * 0.5,
            200.0,
        );
        assert_eq!(
            canvas.count_pixels(&pixels, code, |c| near(c, theme.chrome.accent_solid)),
            0
        );
    }
}
