use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// Count lines in a file efficiently using buffered reading.
pub fn count_lines(path: &Path) -> usize {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    BufReader::new(file).lines().count()
}

/// Find the longest function/method in a file.
/// Returns (longest_function_lines, function_count).
pub fn longest_function(path: &Path) -> (usize, usize) {
    let content = match read_file(path) {
        Some(c) => c,
        None => return (0, 0),
    };
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    dispatch_by_language(ext, &content)
}

fn read_file(path: &Path) -> Option<String> {
    let mut content = String::new();
    let file = File::open(path).ok()?;
    BufReader::new(file).read_to_string(&mut content).ok()?;
    Some(content)
}

fn dispatch_by_language(ext: &str, content: &str) -> (usize, usize) {
    match ext {
        "rs" => longest_brace_fn(content, &["fn "]),
        "go" => longest_brace_fn(content, &["func "]),
        "ts" | "tsx" | "js" | "jsx" | "mjs" => {
            longest_brace_fn(content, &["function ", "=> {", "async "])
        }
        "java" | "kt" | "kts" | "cs" | "dart" | "swift" => {
            longest_brace_fn(content, &["fun ", "func ", "void ", "int ", "string "])
        }
        "py" => longest_indented_fn(content),
        "rb" => longest_ruby_fn(content),
        _ => (0, 0),
    }
}

/// Brace-delimited languages: find function start, count to matching close.
fn longest_brace_fn(content: &str, markers: &[&str]) -> (usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let mut max_len = 0;
    let mut fn_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if !is_fn_start(line, markers) {
            continue;
        }
        fn_count += 1;
        let len = count_brace_block(&lines, i);
        if len > max_len {
            max_len = len;
        }
    }
    (max_len, fn_count)
}

fn is_fn_start(line: &str, markers: &[&str]) -> bool {
    let trimmed = line.trim();
    markers.iter().any(|m| trimmed.contains(m))
        && (trimmed.contains('(') || trimmed.contains('{'))
}

fn count_brace_block(lines: &[&str], start: usize) -> usize {
    let initial_depth = brace_depth_at(lines, start);
    let mut depth = initial_depth;
    let mut fn_lines = 0;
    let mut found_open = false;

    for j in start..lines.len() {
        for ch in non_string_chars(lines[j]) {
            if ch == '{' { depth += 1; found_open = true; }
            else if ch == '}' { depth -= 1; }
        }
        fn_lines += 1;
        if found_open && depth <= initial_depth { break; }
    }
    fn_lines
}

fn brace_depth_at(lines: &[&str], line: usize) -> i32 {
    let mut depth = 0i32;
    for j in 0..line {
        for ch in non_string_chars(lines[j]) {
            if ch == '{' { depth += 1; }
            else if ch == '}' { depth -= 1; }
        }
    }
    depth
}

/// Iterate chars, skipping content inside string/char literals.
fn non_string_chars(line: &str) -> Vec<char> {
    let mut result = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' {
            // Skip until closing quote
            i += 1;
            while i < chars.len() {
                if chars[i] == '"' && (i == 0 || chars[i - 1] != '\\') { break; }
                i += 1;
            }
        } else if ch == '\'' && i + 2 < chars.len() {
            // Skip char literals like '{' or '}'
            if chars.get(i + 2) == Some(&'\'') || chars.get(i + 3) == Some(&'\'') {
                // Skip 'x' or '\x'
                i += if chars.get(i + 1) == Some(&'\\') { 3 } else { 2 };
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
        i += 1;
    }
    result
}

/// Python: `def` with indentation-based blocks.
fn longest_indented_fn(content: &str) -> (usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let mut max_len = 0;
    let mut fn_count = 0;
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();
        if !trimmed.starts_with("def ") && !trimmed.starts_with("async def ") {
            i += 1;
            continue;
        }
        fn_count += 1;
        let len = count_indent_block(&lines, i);
        if len > max_len { max_len = len; }
        i += len;
    }
    (max_len, fn_count)
}

fn count_indent_block(lines: &[&str], start: usize) -> usize {
    let base_indent = lines[start].len() - lines[start].trim_start().len();
    let mut fn_lines = 1;
    let mut j = start + 1;
    while j < lines.len() {
        let line = lines[j];
        if line.trim().is_empty() { fn_lines += 1; j += 1; continue; }
        let indent = line.len() - line.trim_start().len();
        if indent <= base_indent { break; }
        fn_lines += 1;
        j += 1;
    }
    fn_lines
}

/// Ruby: `def` ... `end` blocks.
fn longest_ruby_fn(content: &str) -> (usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let mut max_len = 0;
    let mut fn_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if !line.trim().starts_with("def ") { continue; }
        fn_count += 1;
        let len = count_ruby_def(&lines, i);
        if len > max_len { max_len = len; }
    }
    (max_len, fn_count)
}

fn count_ruby_def(lines: &[&str], start: usize) -> usize {
    let base_indent = lines[start].len() - lines[start].trim_start().len();
    let mut fn_lines = 1;
    for j in (start + 1)..lines.len() {
        fn_lines += 1;
        let indent = lines[j].len() - lines[j].trim_start().len();
        if lines[j].trim() == "end" && indent == base_indent { break; }
    }
    fn_lines
}
