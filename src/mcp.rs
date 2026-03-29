use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::Result;
use serde_json::{json, Value};

use crate::{detection, rules, scan, scoring};

/// Run as an MCP server over stdio (JSON-RPC 2.0).
pub fn serve() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }

        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let response = handle_request(&request);
        let out = serde_json::to_string(&response)?;
        writeln!(stdout, "{out}")?;
        stdout.flush()?;
    }
    Ok(())
}

fn handle_request(req: &Value) -> Value {
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string();
    let id = req.get("id").cloned().unwrap_or(json!(null));

    match method.as_str() {
        "initialize" => {
            let result = json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {"name": "claude-native", "version": "0.1.0"},
                "capabilities": {"tools": {}}
            });
            json_rpc_result(id, result)
        }
        "tools/list" => {
            let result = json!({"tools": tool_definitions()});
            json_rpc_result(id, result)
        }
        "tools/call" => handle_tool_call(id, req),
        "notifications/initialized" => json!(null),
        _ => json_rpc_error(id, -32601, "Method not found"),
    }
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "score",
            "description": "Score how Claude Native a project is (0-100)",
            "inputSchema": {
                "type": "object",
                "properties": {"path": {"type": "string", "description": "Project directory"}},
                "required": ["path"]
            }
        }),
        json!({
            "name": "suggest",
            "description": "Get suggestions to improve Claude Native score",
            "inputSchema": {
                "type": "object",
                "properties": {"path": {"type": "string", "description": "Project directory"}},
                "required": ["path"]
            }
        }),
    ]
}

fn handle_tool_call(id: Value, req: &Value) -> Value {
    let empty = json!({});
    let params = req.get("params").unwrap_or(&empty);
    let tool = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let args = params.get("arguments").unwrap_or(&empty);
    let path = args.get("path").and_then(|p| p.as_str()).unwrap_or(".");

    match tool {
        "score" => run_score(id, path),
        "suggest" => run_suggest(id, path),
        _ => json_rpc_error(id, -32602, "Unknown tool"),
    }
}

fn run_score(id: Value, path: &str) -> Value {
    match score_project(path) {
        Ok(sc) => json_rpc_result(id, json!({
            "content": [{
                "type": "text",
                "text": format!(
                    "Score: {:.0}/100 ({})\nProject Type: {}\n\nDimensions:\n{}",
                    sc.total_score, sc.grade, sc.project_type,
                    sc.dimensions.iter()
                        .map(|d| format!("  {}: {:.0}/100", d.dimension, d.score))
                        .collect::<Vec<_>>().join("\n")
                )
            }]
        })),
        Err(e) => json_rpc_result(id, json!({
            "content": [{"type": "text", "text": format!("Error: {e}")}],
            "isError": true
        })),
    }
}

fn run_suggest(id: Value, path: &str) -> Value {
    match score_project(path) {
        Ok(sc) => {
            let suggestions: Vec<String> = sc.suggestions.iter().enumerate()
                .map(|(i, s)| format!("{}. [{}] {} — {}", i + 1, s.priority, s.title, s.effort))
                .collect();
            let text = if suggestions.is_empty() {
                "No suggestions — project is well-configured!".into()
            } else {
                format!("Suggestions ({}):\n{}", suggestions.len(), suggestions.join("\n"))
            };
            json_rpc_result(id, json!({"content": [{"type": "text", "text": text}]}))
        }
        Err(e) => json_rpc_result(id, json!({
            "content": [{"type": "text", "text": format!("Error: {e}")}],
            "isError": true
        })),
    }
}

fn score_project(path: &str) -> Result<scoring::Scorecard> {
    let path = PathBuf::from(path).canonicalize()?;
    let mut ctx = scan::build_context(&path)?;
    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());
    let results: Vec<_> = rules::all_rules().iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();
    Ok(scoring::calculate(results, &pt))
}

fn json_rpc_result(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

fn json_rpc_error(id: Value, code: i32, msg: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": msg}})
}
