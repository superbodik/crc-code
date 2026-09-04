use crc_agent::{Event, Speaker};
use crc_ui::geometry::Rect;
use crc_ui::view::agent::{self, AgentView, Target};

fn aside() -> Rect {
    Rect::new(1120.0, 40.0, 320.0, 820.0)
}

fn view() -> AgentView {
    AgentView::default()
}

mod what_it_says {
    use super::*;

    #[test]
    fn a_missing_cli_is_said_plainly_and_blocks_sending() {
        let mut state = view();
        state.missing = true;
        state.draft = "hello".to_string();

        assert_eq!(state.status(), "claude не найден в PATH");
        assert!(!state.ready_to_send());
        assert!(!state.stoppable());
        assert!(state.greeting().contains("Поставь"));
    }

    #[test]
    fn an_empty_draft_is_not_worth_sending() {
        let mut state = view();
        assert!(!state.ready_to_send());

        state.draft = "   \n  ".to_string();
        assert!(!state.ready_to_send(), "whitespace is not a question");

        state.draft = "why?".to_string();
        assert!(state.ready_to_send());
    }

    #[test]
    fn a_busy_agent_cannot_be_asked_again_but_can_be_stopped() {
        let mut state = view();
        state.draft = "why?".to_string();
        state.talk.asked("first");

        assert!(!state.ready_to_send());
        assert!(state.stoppable());
        assert_eq!(state.status(), "думает...");
    }

    #[test]
    fn the_model_stands_in_for_a_status_once_it_is_known() {
        let mut state = view();
        state.talk.take(Event::Ready {
            session: "abc".to_string(),
            model: "claude-opus-5".to_string(),
            tools: 0,
        });
        state.talk.note = None;

        assert_eq!(state.status(), "claude-opus-5");
    }

    #[test]
    fn the_open_file_is_shown_rather_than_slipped_in_unseen() {
        let mut state = view();
        assert_eq!(state.context_note(), None);

        state.context = Some("app.rs".to_string());
        assert_eq!(state.context_note().as_deref(), Some("в работе: app.rs"));
    }
}

mod placing {
    use super::*;

    #[test]
    fn everything_stacks_down_the_column_without_overlapping() {
        let placed = agent::layout(aside(), 1.0);

        assert_eq!(placed.header.y, aside().y);
        assert!(placed.header.bottom() <= placed.transcript.y);
        assert!(placed.transcript.bottom() <= placed.status.y);
        assert!(placed.status.bottom() <= placed.composer.y);
        assert!(placed.composer.bottom() <= aside().bottom());
    }

    #[test]
    fn the_close_and_send_buttons_stay_inside_the_column() {
        let placed = agent::layout(aside(), 1.0);

        for rect in [placed.close, placed.send] {
            assert!(rect.x >= aside().x);
            assert!(rect.right() <= aside().right());
        }
        assert!(placed.send.bottom() <= placed.composer.bottom());
    }

    #[test]
    fn it_scales_with_the_display() {
        let one = agent::layout(aside(), 1.0);
        let two = agent::layout(aside(), 2.0);

        assert!(two.header.height > one.header.height);
        assert!(two.composer.height > one.composer.height);
    }
}

mod pressing {
    use super::*;

    #[test]
    fn the_same_button_sends_when_idle_and_stops_when_busy() {
        let placed = agent::layout(aside(), 1.0);
        let idle = view();
        let mut busy = view();
        busy.talk.asked("hello");

        let point = (placed.send.x + 4.0, placed.send.y + 4.0);
        assert_eq!(
            agent::target_at(&placed, &idle, point.0, point.1),
            Some(Target::Send)
        );
        assert_eq!(
            agent::target_at(&placed, &busy, point.0, point.1),
            Some(Target::Stop),
            "a running turn needs a way out"
        );
    }

    #[test]
    fn the_cross_and_the_composer_answer_to_a_click() {
        let placed = agent::layout(aside(), 1.0);
        let state = view();

        assert_eq!(
            agent::target_at(&placed, &state, placed.close.x + 4.0, placed.close.y + 4.0),
            Some(Target::Close)
        );
        assert_eq!(
            agent::target_at(
                &placed,
                &state,
                placed.composer.x + 20.0,
                placed.composer.y + 10.0
            ),
            Some(Target::Composer)
        );
    }

    #[test]
    fn the_transcript_is_not_a_button() {
        let placed = agent::layout(aside(), 1.0);
        let state = view();

        assert_eq!(
            agent::target_at(
                &placed,
                &state,
                placed.transcript.x + 20.0,
                placed.transcript.y + 40.0
            ),
            None
        );
    }
}

mod wrapping {
    use super::*;

    #[test]
    fn a_short_line_is_left_alone() {
        assert_eq!(agent::wrap("hello there", 40), vec!["hello there"]);
    }

    #[test]
    fn a_long_line_is_broken_between_words() {
        let lines = agent::wrap("one two three four five six seven", 12);

        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.chars().count() <= 12, "too wide: {line}");
        }
        assert_eq!(lines.join(" "), "one two three four five six seven");
    }

    #[test]
    fn a_word_longer_than_the_column_is_cut_rather_than_left_to_spill() {
        let lines = agent::wrap(&"x".repeat(50), 10);

        assert_eq!(lines.len(), 5);
        for line in &lines {
            assert!(line.chars().count() <= 10);
        }
    }

    #[test]
    fn existing_line_breaks_are_kept() {
        let lines = agent::wrap("first\nsecond", 40);
        assert_eq!(lines, vec!["first", "second"]);
    }

    #[test]
    fn cyrillic_is_counted_in_letters_not_bytes() {
        let lines = agent::wrap("привет мир как дела", 10);

        for line in &lines {
            assert!(line.chars().count() <= 10, "too wide: {line}");
        }
    }
}

mod the_transcript {
    use super::*;

    #[test]
    fn a_tool_line_reads_as_the_tool_and_what_it_touched() {
        let mut state = view();
        state.talk.take(Event::Using {
            tool: "Edit".to_string(),
            detail: "src/main.rs".to_string(),
        });

        let turn = state.talk.turns.last().unwrap();
        assert_eq!(turn.speaker, Speaker::Tool);
        assert_eq!(turn.text, "Edit: src/main.rs");
    }

    #[test]
    fn every_speaker_has_a_glyph_of_its_own() {
        let glyphs = [
            agent::glyph(Speaker::You),
            agent::glyph(Speaker::Claude),
            agent::glyph(Speaker::Tool),
            agent::glyph(Speaker::Editor),
        ];

        let mut seen: Vec<char> = glyphs.to_vec();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), glyphs.len(), "two speakers share a glyph");
    }
}
