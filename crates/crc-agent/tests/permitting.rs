use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use crc_agent::permission::{
    Request, Verdict, Warden, answer_of, config, read_verdict, tool_name,
};
use serde_json::{Value, json};

mod the_contract {
    use super::*;

    #[test]
    fn the_tool_is_named_the_way_the_cli_addresses_mcp_tools() {
        assert_eq!(tool_name(), "mcp__crc__approve");
    }

    #[test]
    fn the_config_points_the_cli_back_at_this_editor() {
        let written = config("C:/crc/crc.exe", 51234);
        let parsed: Value = serde_json::from_str(&written).expect("valid json");

        let server = &parsed["mcpServers"]["crc"];
        assert_eq!(server["type"], "stdio");
        assert_eq!(server["command"], "C:/crc/crc.exe");
        assert_eq!(server["args"][0], "--permission-relay");
        assert_eq!(server["args"][1], "51234");
    }

    #[test]
    fn allowing_hands_the_input_back_unchanged() {
        let input = json!({ "file_path": "src/main.rs", "content": "fn main() {}" });
        let answer = answer_of(&Verdict::Allow, &input);

        assert_eq!(answer["behavior"], "allow");
        assert_eq!(
            answer["updatedInput"], input,
            "the tool must run with what it asked for"
        );
    }

    #[test]
    fn denying_carries_a_reason_and_no_input() {
        let answer = answer_of(&Verdict::Deny("нет".to_string()), &json!({}));

        assert_eq!(answer["behavior"], "deny");
        assert_eq!(answer["message"], "нет");
        assert!(answer.get("updatedInput").is_none());
    }

    #[test]
    fn a_verdict_travels_as_one_line() {
        let allow = Verdict::Allow.to_line(7);
        assert!(allow.ends_with('\n'));
        assert_eq!(allow.matches('\n').count(), 1);

        let parsed: Value = serde_json::from_str(allow.trim()).unwrap();
        assert_eq!(parsed["id"], 7);
        assert_eq!(parsed["behavior"], "allow");
    }

    #[test]
    fn anything_that_is_not_a_clear_allow_is_a_refusal() {
        assert_eq!(read_verdict(r#"{"behavior":"allow"}"#), Verdict::Allow);
        assert!(matches!(
            read_verdict(r#"{"behavior":"deny","message":"нельзя"}"#),
            Verdict::Deny(why) if why == "нельзя"
        ));
        assert!(
            matches!(read_verdict("not json at all"), Verdict::Deny(_)),
            "a garbled answer must never be read as permission"
        );
        assert!(
            matches!(read_verdict("{}"), Verdict::Deny(_)),
            "silence is not consent"
        );
    }
}

mod what_is_being_asked {
    use super::*;

    fn request(input: Value) -> Request {
        Request {
            id: 1,
            tool: "Edit".to_string(),
            input,
        }
    }

    #[test]
    fn the_file_is_found_wherever_the_tool_put_it() {
        assert_eq!(
            request(json!({ "file_path": "a.rs" })).file().as_deref(),
            Some("a.rs")
        );
        assert_eq!(
            request(json!({ "path": "b.rs" })).file().as_deref(),
            Some("b.rs")
        );
        assert_eq!(
            request(json!({ "notebook_path": "c.ipynb" })).file().as_deref(),
            Some("c.ipynb")
        );
        assert_eq!(request(json!({ "command": "ls" })).file(), None);
    }

    #[test]
    fn the_summary_names_the_tool_and_the_file_it_wants() {
        assert_eq!(
            request(json!({ "file_path": "src/main.rs" })).summary(),
            "Edit · src/main.rs"
        );
        assert_eq!(request(json!({ "command": "ls" })).summary(), "Edit");
    }
}

mod over_the_wire {
    use super::*;

    fn wait_for<T>(mut probe: impl FnMut() -> Option<T>, patience: Duration) -> Option<T> {
        let started = Instant::now();
        while started.elapsed() < patience {
            if let Some(found) = probe() {
                return Some(found);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    }

    #[test]
    fn a_question_reaches_the_editor_and_the_answer_comes_back() {
        let warden = Warden::listen().expect("a door");

        let stream = TcpStream::connect(SocketAddr::from((Ipv4Addr::LOCALHOST, warden.port())))
            .expect("the relay can knock");
        let mut writing = stream.try_clone().unwrap();
        let mut answers = BufReader::new(stream).lines();

        let asked = json!({
            "id": 4,
            "tool": "Write",
            "input": { "file_path": "notes.txt", "content": "hello" }
        });
        writing
            .write_all(format!("{asked}\n").as_bytes())
            .expect("the question goes out");
        writing.flush().unwrap();

        let request = wait_for(|| warden.waiting(), Duration::from_secs(5))
            .expect("the editor never saw the question");

        assert_eq!(request.id, 4);
        assert_eq!(request.tool, "Write");
        assert_eq!(request.file().as_deref(), Some("notes.txt"));

        warden.answer(request.id, Verdict::Allow);

        let answer = answers
            .next()
            .expect("the relay never heard back")
            .expect("a readable line");
        let parsed: Value = serde_json::from_str(&answer).unwrap();

        assert_eq!(parsed["id"], 4);
        assert_eq!(parsed["behavior"], "allow");
    }

    #[test]
    fn a_refusal_travels_with_its_reason() {
        let warden = Warden::listen().expect("a door");

        let stream = TcpStream::connect(SocketAddr::from((Ipv4Addr::LOCALHOST, warden.port())))
            .expect("the relay can knock");
        let mut writing = stream.try_clone().unwrap();
        let mut answers = BufReader::new(stream).lines();

        writing
            .write_all(b"{\"id\":9,\"tool\":\"Bash\",\"input\":{\"command\":\"rm -rf /\"}}\n")
            .unwrap();
        writing.flush().unwrap();

        let request = wait_for(|| warden.waiting(), Duration::from_secs(5)).expect("a question");
        warden.answer(request.id, Verdict::Deny("ни за что".to_string()));

        let answer = answers.next().unwrap().unwrap();
        let parsed: Value = serde_json::from_str(&answer).unwrap();

        assert_eq!(parsed["behavior"], "deny");
        assert_eq!(parsed["message"], "ни за что");
    }

    #[test]
    fn every_warden_gets_a_door_of_its_own() {
        let first = Warden::listen().expect("a door");
        let second = Warden::listen().expect("another door");

        assert_ne!(first.port(), second.port());
        assert!(first.port() > 0 && second.port() > 0);
    }

    #[test]
    fn nothing_is_waiting_until_something_asks() {
        let warden = Warden::listen().expect("a door");
        assert!(warden.waiting().is_none());
    }
}
