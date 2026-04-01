use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

#[test]
fn prompt_json_with_tool_use_writes_clean_transport_output() {
    let fixture_root = unique_temp_dir("claw-json-transport");
    fs::create_dir_all(&fixture_root).expect("create fixture root");
    fs::write(fixture_root.join("fixture.txt"), "fixture contents\n").expect("write fixture file");
    fs::create_dir_all(fixture_root.join("config")).expect("create config dir");

    let server = TestServer::spawn(vec![
        sse_response(
            "req_tool",
            &tool_use_stream("read_file", json!({ "path": "fixture.txt" })),
        ),
        sse_response("req_done", &text_stream("done")),
    ]);

    let output = Command::new(env!("CARGO_BIN_EXE_claw"))
        .current_dir(&fixture_root)
        .env("ANTHROPIC_BASE_URL", server.base_url())
        .env("ANTHROPIC_API_KEY", "test-key")
        .env("CLAW_CONFIG_HOME", fixture_root.join("config"))
        .arg("--output-format")
        .arg("json")
        .arg("prompt")
        .arg("use a tool")
        .output()
        .expect("run claw prompt json");

    server.finish();

    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");

    assert!(
        output.status.success(),
        "status: {:?}\nstderr:\n{stderr}",
        output.status
    );
    assert!(stderr.trim().is_empty(), "unexpected stderr: {stderr}");
    assert!(
        stdout.trim_start().starts_with('{'),
        "stdout should begin with JSON object, got:\n{stdout}"
    );

    let parsed: Value = serde_json::from_str(stdout.trim())
        .expect("full stdout should be a single parseable JSON object");

    assert_eq!(parsed["message"], "done");
    assert_eq!(parsed["iterations"], 2);
    assert_eq!(parsed["tool_uses"].as_array().map(Vec::len), Some(1));
    assert_eq!(parsed["tool_results"].as_array().map(Vec::len), Some(1));
    assert_eq!(parsed["tool_uses"][0]["name"], "read_file");
    assert_eq!(parsed["tool_results"][0]["tool_name"], "read_file");
    assert_eq!(parsed["tool_results"][0]["is_error"], false);

    let tool_output = parsed["tool_results"][0]["output"]
        .as_str()
        .expect("tool result output string");
    assert!(tool_output.contains("fixture contents"));
    assert!(
        !stdout.contains("📄 Read"),
        "stdout leaked human-readable tool rendering:\n{stdout}"
    );
}

struct TestServer {
    base_url: String,
    join_handle: thread::JoinHandle<()>,
}

impl TestServer {
    fn spawn(responses: Vec<String>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        listener
            .set_nonblocking(true)
            .expect("set nonblocking listener");
        let address = listener.local_addr().expect("listener addr");
        let join_handle = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut served = 0usize;

            while served < responses.len() && Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        drain_http_request(&mut stream);
                        stream
                            .write_all(responses[served].as_bytes())
                            .expect("write response");
                        served += 1;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("accept failed: {error}"),
                }
            }

            assert_eq!(
                served,
                responses.len(),
                "server did not observe expected request count"
            );
        });

        Self {
            base_url: format!("http://{address}"),
            join_handle,
        }
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn finish(self) {
        self.join_handle.join().expect("join server thread");
    }
}

fn drain_http_request(stream: &mut std::net::TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("set read timeout");
    let mut buffer = Vec::new();
    let mut header_end = None;

    while header_end.is_none() {
        let mut chunk = [0_u8; 1024];
        let read = stream.read(&mut chunk).expect("read request chunk");
        if read == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read]);
        header_end = find_header_end(&buffer);
    }

    let header_end = header_end.expect("request should contain headers");
    let headers = String::from_utf8(buffer[..header_end].to_vec()).expect("header utf8");
    let content_length = headers
        .lines()
        .find_map(|line| {
            line.split_once(':').and_then(|(name, value)| {
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("content length"))
            })
        })
        .unwrap_or(0);
    let mut body = buffer[(header_end + 4)..].to_vec();
    while body.len() < content_length {
        let mut chunk = vec![0_u8; content_length - body.len()];
        let read = stream.read(&mut chunk).expect("read request body");
        if read == 0 {
            break;
        }
        body.extend_from_slice(&chunk[..read]);
    }
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn sse_response(request_id: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nrequest-id: {request_id}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn tool_use_stream(tool_name: &str, input: Value) -> String {
    let mut body = String::new();
    body.push_str(&sse_event(
        "message_start",
        json!({
            "type": "message_start",
            "message": {
                "id": "msg_tool",
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": "claude-opus-4-6",
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {"input_tokens": 8, "output_tokens": 0}
            }
        }),
    ));
    body.push_str(&sse_event(
        "content_block_start",
        json!({
            "type": "content_block_start",
            "index": 0,
            "content_block": {
                "type": "tool_use",
                "id": "toolu_1",
                "name": tool_name,
                "input": {}
            }
        }),
    ));
    body.push_str(&sse_event(
        "content_block_delta",
        json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "input_json_delta",
                "partial_json": input.to_string()
            }
        }),
    ));
    body.push_str(&sse_event(
        "content_block_stop",
        json!({"type": "content_block_stop", "index": 0}),
    ));
    body.push_str(&sse_event(
        "message_delta",
        json!({
            "type": "message_delta",
            "delta": {"stop_reason": "tool_use", "stop_sequence": null},
            "usage": {"input_tokens": 8, "output_tokens": 1}
        }),
    ));
    body.push_str(&sse_event("message_stop", json!({"type": "message_stop"})));
    body.push_str("data: [DONE]\n\n");
    body
}

fn text_stream(text: &str) -> String {
    let mut body = String::new();
    body.push_str(&sse_event(
        "message_start",
        json!({
            "type": "message_start",
            "message": {
                "id": "msg_done",
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": "claude-opus-4-6",
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {"input_tokens": 20, "output_tokens": 0}
            }
        }),
    ));
    body.push_str(&sse_event(
        "content_block_start",
        json!({
            "type": "content_block_start",
            "index": 0,
            "content_block": {"type": "text", "text": ""}
        }),
    ));
    body.push_str(&sse_event(
        "content_block_delta",
        json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {"type": "text_delta", "text": text}
        }),
    ));
    body.push_str(&sse_event(
        "content_block_stop",
        json!({"type": "content_block_stop", "index": 0}),
    ));
    body.push_str(&sse_event(
        "message_delta",
        json!({
            "type": "message_delta",
            "delta": {"stop_reason": "end_turn", "stop_sequence": null},
            "usage": {"input_tokens": 20, "output_tokens": 2}
        }),
    ));
    body.push_str(&sse_event("message_stop", json!({"type": "message_stop"})));
    body.push_str("data: [DONE]\n\n");
    body
}

fn sse_event(event_name: &str, payload: Value) -> String {
    format!("event: {event_name}\ndata: {payload}\n\n")
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nanos}"))
}
