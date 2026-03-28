mod helpers;

use claude_native::token_cost;

#[test]
fn estimates_savings_for_missing_claudeignore() {
    let estimate = token_cost::estimate_savings("1.5", &helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
    ]).1);
    // No noise dirs → returns None or 0 estimate
    assert!(estimate.is_none() || estimate.unwrap().contains("0"));
}

#[test]
fn estimates_savings_for_missing_claude_md() {
    let estimate = token_cost::estimate_savings("1.1", &helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
    ]).1);
    assert!(estimate.is_some());
    assert!(estimate.unwrap().contains("token"));
}

#[test]
fn estimates_savings_for_lock_files() {
    let big_lock = "# lock\n".repeat(500);
    let estimate = token_cost::estimate_savings("2.3", &helpers::scan_project(&[
        ("Cargo.lock", &big_lock),
        ("src/main.rs", "fn main() {}"),
    ]).1);
    assert!(estimate.is_some());
    assert!(estimate.unwrap().contains("500"));
}
