use crc_agent::event::{Event, read};
use crc_agent::session::{arguments, ask};
use crc_agent::{Speaker, Talk};

fn one(line: &str) -> Event {
    let mut events = read(line);
    assert_eq!(events.len(), 1, "expected exactly one event from {line}");
    events.remove(0)
}

mod reading_the_stream {
    use super::*;

    #[test]
    fn the_opening_line_says_who_answered() {
        let line = r#"{"type":"system","subtype":"init","session_id":"abc","model":"claude-opus-5","tools":["Read","Edit","Bash"]}"#;

        assert_eq!(
            one(line),
            Event::Ready {
                session: "abc".to_string(),
                model: "claude-opus-5".to_string(),
                tools: 3,
            }
        );
    }

    #[test]
    fn other_system_chatter_is_ignored() {
        let line = r#"{"type":"system","subtype":"thinking_tokens","estimated_tokens":95}"#;
        assert!(read(line).is_empty(), "token counters are not conversation");
    }

    #[test]
    fn plain_text_comes_through_as_something_said() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"pong"}]}}"#;
        assert_eq!(one(line), Event::Said("pong".to_string()));
    }

    #[test]
    fn thinking_is_kept_apart_from_speech() {
        let line = r#"{"type":"assistant","message":{"content":[{"type":"thinking","thinking":"weighing it up"}]}}"#;
        assert_eq!(one(line), Event::Thought("weighing it up".to_string()));
    }

    #[test]
    fn a_message_with_several_blocks_yields_several_events() {
        let line = r#"{"type":"assistant","message":{"content":[
            {"type":"thinking","thinking":"hm"},
            {"type":"text","text":"here goes"},
            {"type":"tool_use","name":"Read","input":{"file_path":"src/main.rs"}}
        ]}}"#;

        let events = read(line);
        assert_eq!(events.len(), 3);
        assert_eq!(events[1], Event::Said("here goes".to_string()));
        assert_eq!(
            events[2],
            Event::Using {
                tool: "Read".to_string(),
                detail: "src/main.rs".to_string(),
            }
        );
    }

    #[test]
    fn a_tool_is_named_by_the_part_of_its_input_a_person_would_read() {
        let bash = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"cargo test","description":"run the tests"}}]}}"#;
        assert_eq!(
            one(bash),
            Event::Using {
                tool: "Bash".to_string(),
                detail: "cargo test".to_string(),
            }
        );

        let bare = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"TodoWrite","input":{"todos":[]}}]}}"#;
        assert_eq!(
            one(bare),
            Event::Using {
                tool: "TodoWrite".to_string(),
                detail: String::new(),
            }
        );
    }

    #[test]
    fn a_long_command_is_cut_down_to_one_line() {
        let long = "x".repeat(400);
        let line = format!(
            r#"{{"type":"assistant","message":{{"content":[{{"type":"tool_use","name":"Bash","input":{{"command":"{long}"}}}}]}}}}"#
        );

        match one(&line) {
            Event::Using { detail, .. } => {
                assert!(detail.chars().count() <= 124, "too long: {}", detail.len());
                assert!(detail.ends_with("..."));
            }
            other => panic!("expected a tool use, got {other:?}"),
        }
    }

    #[test]
    fn a_tool_result_reports_whether_it_went_wrong() {
        let good = r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t1"}]}}"#;
        assert_eq!(
            one(good),
            Event::Returned {
                tool: "t1".to_string(),
                trouble: false,
            }
        );

        let bad = r#"{"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"t2","is_error":true}]}}"#;
        assert_eq!(
            one(bad),
            Event::Returned {
                tool: "t2".to_string(),
                trouble: true,
            }
        );
    }

    #[test]
    fn the_closing_line_carries_the_cost() {
        let line = r#"{"type":"result","subtype":"success","is_error":false,"result":"done","num_turns":3,"total_cost_usd":0.0531}"#;

        assert_eq!(
            one(line),
            Event::Finished {
                text: "done".to_string(),
                cost: 0.0531,
                turns: 3,
            }
        );
    }

    #[test]
    fn a_failed_run_is_reported_as_trouble_not_as_a_finish() {
        let line = r#"{"type":"result","subtype":"error_max_turns","is_error":true,"result":"ran out of turns"}"#;
        assert_eq!(one(line), Event::Trouble("ran out of turns".to_string()));
    }

    #[test]
    fn a_rate_limit_is_only_worth_saying_when_it_bites() {
        let line = r#"{"type":"rate_limit_event","rate_limit_info":{"status":"allowed","resetsAt":1788562800}}"#;
        assert_eq!(
            one(line),
            Event::Limited {
                status: "allowed".to_string(),
                resets_at: Some(1788562800),
            }
        );
    }

    #[test]
    fn a_line_that_is_not_json_is_kept_as_trouble_rather_than_dropped() {
        assert_eq!(
            one("something went very wrong"),
            Event::Trouble("something went very wrong".to_string())
        );
    }

    #[test]
    fn blank_lines_are_nothing_at_all() {
        assert!(read("").is_empty());
        assert!(read("   \n").is_empty());
    }
}

mod speaking {
    use super::*;

    #[test]
    fn a_question_goes_out_as_one_json_line() {
        let line = ask("what does this do?");

        assert!(line.ends_with('\n'), "the agent reads line by line");
        assert_eq!(line.matches('\n').count(), 1);

        let parsed: serde_json::Value = serde_json::from_str(line.trim()).expect("valid json");
        assert_eq!(parsed["type"], "user");
        assert_eq!(parsed["message"]["role"], "user");
        assert_eq!(parsed["message"]["content"][0]["text"], "what does this do?");
    }

    #[test]
    fn a_question_with_newlines_and_quotes_survives_the_trip() {
        let awkward = "fix this:\n\"let x = 1\"\n\tand this";
        let line = ask(awkward);

        assert_eq!(line.matches('\n').count(), 1, "it must stay one line");

        let parsed: serde_json::Value = serde_json::from_str(line.trim()).expect("valid json");
        assert_eq!(parsed["message"]["content"][0]["text"], awkward);
    }

    #[test]
    fn the_flags_ask_for_a_stream_in_both_directions() {
        let flags = arguments(None, None);

        assert!(flags.contains(&"-p".to_string()));
        assert!(flags.windows(2).any(|pair| pair
            == ["--input-format".to_string(), "stream-json".to_string()]));
        assert!(flags.windows(2).any(|pair| pair
            == ["--output-format".to_string(), "stream-json".to_string()]));
        assert!(!flags.contains(&"--model".to_string()));
    }

    #[test]
    fn a_chosen_model_is_passed_through() {
        let flags = arguments(Some("claude-haiku-4-5-20251001"), None);

        assert!(flags.windows(2).any(|pair| pair
            == [
                "--model".to_string(),
                "claude-haiku-4-5-20251001".to_string()
            ]));
    }
}

mod the_conversation {
    use super::*;

    #[test]
    fn asking_marks_the_talk_busy_until_the_answer_lands() {
        let mut talk = Talk::default();
        talk.asked("hello");

        assert!(talk.busy);
        assert_eq!(talk.turns.len(), 1);
        assert_eq!(talk.turns[0].speaker, Speaker::You);

        talk.take(Event::Said("hi".to_string()));
        assert!(talk.busy, "text alone does not end the turn");

        talk.take(Event::Finished {
            text: "hi".to_string(),
            cost: 0.01,
            turns: 1,
        });
        assert!(!talk.busy);
    }

    #[test]
    fn the_opening_line_fills_in_who_is_answering() {
        let mut talk = Talk::default();
        talk.take(Event::Ready {
            session: "abc".to_string(),
            model: "claude-opus-5".to_string(),
            tools: 12,
        });

        assert_eq!(talk.session, "abc");
        assert_eq!(talk.model, "claude-opus-5");
        assert!(talk.alive);
    }

    #[test]
    fn cost_adds_up_across_answers() {
        let mut talk = Talk::default();
        for _ in 0..3 {
            talk.take(Event::Finished {
                text: String::new(),
                cost: 0.02,
                turns: 1,
            });
        }

        assert!((talk.cost - 0.06).abs() < 1e-9);
    }

    #[test]
    fn a_tool_that_failed_is_marked_on_the_line_that_ran_it() {
        let mut talk = Talk::default();
        talk.take(Event::Using {
            tool: "Bash".to_string(),
            detail: "cargo test".to_string(),
        });
        talk.take(Event::Returned {
            tool: "t1".to_string(),
            trouble: true,
        });

        assert!(talk.turns.last().unwrap().text.ends_with("не вышло"));
    }

    #[test]
    fn a_tool_that_worked_leaves_its_line_alone() {
        let mut talk = Talk::default();
        talk.take(Event::Using {
            tool: "Read".to_string(),
            detail: "src/main.rs".to_string(),
        });
        talk.take(Event::Returned {
            tool: "t1".to_string(),
            trouble: false,
        });

        assert_eq!(talk.turns.last().unwrap().text, "Read: src/main.rs");
    }

    #[test]
    fn thinking_never_reaches_the_transcript() {
        let mut talk = Talk::default();
        talk.take(Event::Thought("weighing it up".to_string()));

        assert!(talk.is_empty(), "thinking is not part of the conversation");
    }

    #[test]
    fn the_agent_going_away_stops_the_wait() {
        let mut talk = Talk::default();
        talk.asked("hello");
        talk.take(Event::Gone);

        assert!(!talk.busy);
        assert!(!talk.alive);
        assert!(talk.note.is_some());
    }

    #[test]
    fn trouble_is_shown_as_the_editor_speaking_not_as_claude() {
        let mut talk = Talk::default();
        talk.asked("hello");
        talk.take(Event::Trouble("the agent could not start".to_string()));

        assert!(!talk.busy);
        assert_eq!(talk.turns.last().unwrap().speaker, Speaker::Editor);
    }

    #[test]
    fn a_limit_that_still_allows_the_call_says_nothing() {
        let mut talk = Talk::default();
        talk.take(Event::Limited {
            status: "allowed".to_string(),
            resets_at: None,
        });

        assert_eq!(talk.note, None);
    }
}

mod finding_the_cli {
    use std::path::Path;

    use crc_agent::session::{candidates, locate, needs_a_shell};

    #[test]
    fn windows_is_offered_the_names_npm_actually_writes() {
        let names = candidates();

        if cfg!(windows) {
            assert!(
                names.contains(&"claude.cmd".to_string()),
                "npm installs a .cmd shim on Windows and nothing else is executable"
            );
            assert!(names.contains(&"claude.exe".to_string()));
        } else {
            assert_eq!(names, vec!["claude".to_string()]);
        }
    }

    #[test]
    fn a_batch_shim_has_to_go_through_the_shell() {
        assert!(needs_a_shell(Path::new("C:/npm/claude.cmd")));
        assert!(needs_a_shell(Path::new("C:/npm/claude.BAT")));
        assert!(!needs_a_shell(Path::new("C:/npm/claude.exe")));
        assert!(!needs_a_shell(Path::new("/usr/local/bin/claude")));
    }

    #[test]
    fn the_search_looks_at_files_and_not_at_folders() {
        if let Some(found) = locate() {
            assert!(found.is_file(), "{} is not a file", found.display());
        }
    }
}

mod counting_in_russian {
    use crc_agent::talk::moves;

    #[test]
    fn one_turn_is_not_plural() {
        assert_eq!(moves(1), "ход");
        assert_eq!(moves(21), "ход");
        assert_eq!(moves(101), "ход");
    }

    #[test]
    fn a_few_turns_take_the_middle_form() {
        for count in [2, 3, 4, 22, 33, 104] {
            assert_eq!(moves(count), "хода", "{count}");
        }
    }

    #[test]
    fn many_turns_take_the_plural() {
        for count in [0, 5, 9, 10, 25, 100] {
            assert_eq!(moves(count), "ходов", "{count}");
        }
    }

    #[test]
    fn the_teens_are_the_exception_they_always_are() {
        for count in [11, 12, 13, 14, 111, 112] {
            assert_eq!(moves(count), "ходов", "{count}");
        }
    }
}
