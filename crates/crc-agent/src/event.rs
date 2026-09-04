use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Ready {
        session: String,
        model: String,
        tools: usize,
    },
    Thought(String),
    Said(String),
    Using {
        tool: String,
        detail: String,
    },
    Returned {
        tool: String,
        trouble: bool,
    },
    Finished {
        text: String,
        cost: f64,
        turns: u64,
    },
    Limited {
        status: String,
        resets_at: Option<u64>,
    },
    Trouble(String),
    Gone,
}

pub fn read(line: &str) -> Vec<Event> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return vec![Event::Trouble(line.to_string())];
    };

    match value.get("type").and_then(Value::as_str) {
        Some("system") => system(&value),
        Some("assistant") => assistant(&value),
        Some("user") => user(&value),
        Some("result") => result(&value),
        Some("rate_limit_event") => limit(&value),
        _ => Vec::new(),
    }
}

fn system(value: &Value) -> Vec<Event> {
    if value.get("subtype").and_then(Value::as_str) != Some("init") {
        return Vec::new();
    }

    vec![Event::Ready {
        session: text_at(value, "session_id"),
        model: text_at(value, "model"),
        tools: value
            .get("tools")
            .and_then(Value::as_array)
            .map(Vec::len)
            .unwrap_or(0),
    }]
}

fn assistant(value: &Value) -> Vec<Event> {
    let Some(blocks) = value
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    let mut events = Vec::new();
    for block in blocks {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                let said = text_at(block, "text");
                if !said.is_empty() {
                    events.push(Event::Said(said));
                }
            }
            Some("thinking") => {
                let thought = text_at(block, "thinking");
                if !thought.is_empty() {
                    events.push(Event::Thought(thought));
                }
            }
            Some("tool_use") => events.push(Event::Using {
                tool: text_at(block, "name"),
                detail: detail_of(block.get("input")),
            }),
            _ => {}
        }
    }
    events
}

fn user(value: &Value) -> Vec<Event> {
    let Some(blocks) = value
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    blocks
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("tool_result"))
        .map(|block| Event::Returned {
            tool: text_at(block, "tool_use_id"),
            trouble: block
                .get("is_error")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        })
        .collect()
}

fn result(value: &Value) -> Vec<Event> {
    if value.get("is_error").and_then(Value::as_bool) == Some(true) {
        let complaint = text_at(value, "result");
        return vec![Event::Trouble(if complaint.is_empty() {
            text_at(value, "subtype")
        } else {
            complaint
        })];
    }

    vec![Event::Finished {
        text: text_at(value, "result"),
        cost: value
            .get("total_cost_usd")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),
        turns: value.get("num_turns").and_then(Value::as_u64).unwrap_or(0),
    }]
}

fn limit(value: &Value) -> Vec<Event> {
    let Some(info) = value.get("rate_limit_info") else {
        return Vec::new();
    };

    vec![Event::Limited {
        status: text_at(info, "status"),
        resets_at: info.get("resetsAt").and_then(Value::as_u64),
    }]
}

fn text_at(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn detail_of(input: Option<&Value>) -> String {
    let Some(input) = input.and_then(Value::as_object) else {
        return String::new();
    };

    for key in [
        "file_path",
        "path",
        "pattern",
        "command",
        "query",
        "url",
        "description",
        "prompt",
    ] {
        if let Some(found) = input.get(key).and_then(Value::as_str)
            && !found.is_empty()
        {
            return shorten(found, 120);
        }
    }
    String::new()
}

fn shorten(text: &str, limit: usize) -> String {
    let flat: String = text
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .take(limit)
        .collect();

    if flat.chars().count() < text.chars().count() {
        format!("{flat}...")
    } else {
        flat
    }
}
