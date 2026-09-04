use std::io::{BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender, channel};

use anyhow::{Context, Result};
use serde_json::{Value, json};

pub const SERVER: &str = "crc";
pub const TOOL: &str = "approve";

pub fn tool_name() -> String {
    format!("mcp__{SERVER}__{TOOL}")
}

pub fn config(program: &str, port: u16) -> String {
    json!({
        "mcpServers": {
            SERVER: {
                "type": "stdio",
                "command": program,
                "args": ["--permission-relay", port.to_string()]
            }
        }
    })
    .to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub id: u64,
    pub tool: String,
    pub input: Value,
}

impl Request {
    pub fn file(&self) -> Option<String> {
        for key in ["file_path", "path", "notebook_path"] {
            if let Some(found) = self.input.get(key).and_then(Value::as_str) {
                return Some(found.to_string());
            }
        }
        None
    }

    pub fn summary(&self) -> String {
        match self.file() {
            Some(file) => format!("{} · {file}", self.tool),
            None => self.tool.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Allow,
    Deny(String),
}

impl Verdict {
    pub fn to_line(&self, id: u64) -> String {
        let body = match self {
            Verdict::Allow => json!({ "id": id, "behavior": "allow" }),
            Verdict::Deny(why) => json!({ "id": id, "behavior": "deny", "message": why }),
        };
        format!("{body}\n")
    }
}

pub fn answer_of(verdict: &Verdict, input: &Value) -> Value {
    match verdict {
        Verdict::Allow => json!({ "behavior": "allow", "updatedInput": input }),
        Verdict::Deny(why) => json!({ "behavior": "deny", "message": why }),
    }
}

pub struct Warden {
    port: u16,
    requests: Receiver<Request>,
    answers: Sender<(u64, Verdict)>,
}

impl Warden {
    pub fn listen() -> Result<Self> {
        let listener = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
            .context("could not open a door for the permission relay")?;
        let port = listener.local_addr()?.port();

        let (post, requests) = channel::<Request>();
        let (answers, verdicts) = channel::<(u64, Verdict)>();

        std::thread::Builder::new()
            .name("crc-warden".to_string())
            .spawn(move || {
                for stream in listener.incoming().flatten() {
                    let Ok(reading) = stream.try_clone() else {
                        continue;
                    };
                    let mut writing = stream;
                    let mut lines = BufReader::new(reading).lines();

                    while let Some(Ok(line)) = lines.next() {
                        let Ok(value) = serde_json::from_str::<Value>(&line) else {
                            continue;
                        };
                        let request = Request {
                            id: value.get("id").and_then(Value::as_u64).unwrap_or(0),
                            tool: value
                                .get("tool")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_string(),
                            input: value.get("input").cloned().unwrap_or(Value::Null),
                        };

                        if post.send(request).is_err() {
                            return;
                        }

                        match verdicts.recv() {
                            Ok((id, verdict)) => {
                                if writing.write_all(verdict.to_line(id).as_bytes()).is_err() {
                                    break;
                                }
                                let _ = writing.flush();
                            }
                            Err(_) => return,
                        }
                    }
                }
            })
            .context("could not start the permission warden")?;

        Ok(Self {
            port,
            requests,
            answers,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn waiting(&self) -> Option<Request> {
        self.requests.try_recv().ok()
    }

    pub fn answer(&self, id: u64, verdict: Verdict) {
        let _ = self.answers.send((id, verdict));
    }
}

pub fn relay(port: u16) -> Result<()> {
    let stream = TcpStream::connect(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        .context("the editor is not listening")?;
    let mut editor = stream.try_clone()?;
    let mut answers = BufReader::new(stream).lines();

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut asked = 0u64;

    for line in stdin.lock().lines().map_while(Result::ok) {
        let Ok(message) = serde_json::from_str::<Value>(&line) else {
            continue;
        };

        let method = message
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let id = message.get("id").cloned();

        let reply = match method {
            "initialize" => Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "crc-code", "version": env!("CARGO_PKG_VERSION") }
            })),
            "tools/list" => Some(json!({
                "tools": [{
                    "name": TOOL,
                    "description": "Ask the editor whether a tool may run",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "tool_name": { "type": "string" },
                            "input": { "type": "object" }
                        },
                        "required": ["tool_name", "input"]
                    }
                }]
            })),
            "tools/call" => {
                let arguments = message
                    .get("params")
                    .and_then(|params| params.get("arguments"))
                    .cloned()
                    .unwrap_or(Value::Null);

                let tool = arguments
                    .get("tool_name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let input = arguments.get("input").cloned().unwrap_or(Value::Null);

                asked += 1;
                let question = json!({ "id": asked, "tool": tool, "input": input });
                editor.write_all(format!("{question}\n").as_bytes())?;
                editor.flush()?;

                let verdict = match answers.next() {
                    Some(Ok(answer)) => read_verdict(&answer),
                    _ => Verdict::Deny("редактор закрылся".to_string()),
                };

                Some(json!({
                    "content": [{
                        "type": "text",
                        "text": answer_of(&verdict, &input).to_string()
                    }]
                }))
            }
            _ => None,
        };

        let (Some(id), Some(result)) = (id, reply) else {
            continue;
        };

        let response = json!({ "jsonrpc": "2.0", "id": id, "result": result });
        writeln!(stdout, "{response}")?;
        stdout.flush()?;
    }

    Ok(())
}

pub fn read_verdict(line: &str) -> Verdict {
    let Ok(value) = serde_json::from_str::<Value>(line) else {
        return Verdict::Deny("непонятный ответ редактора".to_string());
    };

    match value.get("behavior").and_then(Value::as_str) {
        Some("allow") => Verdict::Allow,
        _ => Verdict::Deny(
            value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("отклонено")
                .to_string(),
        ),
    }
}
