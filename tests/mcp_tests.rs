mod helpers;

use serde_json::{json, Value};

fn send_mcp(request: &str) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("cargo")
        .args(["run", "--", "--mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to start mcp");

    child.stdin.as_mut().unwrap().write_all(request.as_bytes()).unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().unwrap();
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn mcp_initialize_returns_capabilities() {
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
    let resp = send_mcp(req);
    let v: Value = serde_json::from_str(resp.trim()).unwrap();
    assert_eq!(v["id"], 1);
    assert!(v["result"]["capabilities"].is_object());
    assert_eq!(v["result"]["serverInfo"]["name"], "claude-native");
}

#[test]
fn mcp_tools_list_returns_tools() {
    let req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
    let resp = send_mcp(req);
    let v: Value = serde_json::from_str(resp.trim()).unwrap();
    let tools = v["result"]["tools"].as_array().unwrap();
    assert!(tools.len() >= 2);

    let names: Vec<&str> = tools.iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"score"));
    assert!(names.contains(&"suggest"));
}

#[test]
fn mcp_score_tool_returns_score() {
    let req = format!(
        r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"score","arguments":{{"path":"{}"}}}}}}"#,
        std::env::current_dir().unwrap().display()
    );
    let resp = send_mcp(&req);
    let v: Value = serde_json::from_str(resp.trim()).unwrap();
    let text = v["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Score:"));
    assert!(text.contains("/100"));
}

#[test]
fn mcp_unknown_method_returns_error() {
    let req = r#"{"jsonrpc":"2.0","id":4,"method":"nonexistent"}"#;
    let resp = send_mcp(req);
    let v: Value = serde_json::from_str(resp.trim()).unwrap();
    assert!(v["error"].is_object());
    assert_eq!(v["error"]["code"], -32601);
}
