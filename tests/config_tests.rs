mod helpers;

use claude_native::config::Config;

#[test]
fn config_loads_defaults_without_file() {
    let dir = helpers::create_project(&[
        ("src/main.rs", "fn main() {}"),
    ]);
    let cfg = Config::load(dir.path());
    assert!(cfg.disabled_rules.is_empty());
    assert_eq!(cfg.thresholds.file_lines_error, 500);
    assert_eq!(cfg.thresholds.function_lines_error, 80);
    assert_eq!(cfg.thresholds.claude_md_max_lines, 200);
}

#[test]
fn config_loads_from_yaml() {
    let yaml = r#"
disabled_rules:
  - "4.7"
  - "1.8"
thresholds:
  file_lines_error: 800
  function_lines_error: 100
"#;
    let dir = helpers::create_project(&[
        (".claude-native.yml", yaml),
        ("src/main.rs", "fn main() {}"),
    ]);
    let cfg = Config::load(dir.path());
    assert_eq!(cfg.disabled_rules.len(), 2);
    assert!(cfg.is_rule_disabled("4.7"));
    assert!(cfg.is_rule_disabled("1.8"));
    assert!(!cfg.is_rule_disabled("1.1"));
    assert_eq!(cfg.thresholds.file_lines_error, 800);
    assert_eq!(cfg.thresholds.function_lines_error, 100);
}

#[test]
fn config_disabled_rules_are_skipped() {
    let yaml = "disabled_rules:\n  - \"1.8\"\n";
    let (_dir, mut ctx) = helpers::scan_project(&[
        (".claude-native.yml", yaml),
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let cfg = Config::load(&ctx.root);
    let disabled = cfg.disabled_set();

    let all = claude_native::rules::all_rules();
    let results: Vec<_> = all.iter()
        .filter(|r| r.applies_to(&pt) && !disabled.contains(r.id()))
        .map(|r| r.check(&ctx))
        .collect();

    // Rule 1.8 should not appear in results
    let has_1_8 = results.iter().any(|r| r.rule_id == "1.8");
    assert!(!has_1_8, "Disabled rule 1.8 should not appear in results");
}

#[test]
fn config_handles_invalid_yaml() {
    let dir = helpers::create_project(&[
        (".claude-native.yml", "{{invalid yaml content"),
    ]);
    let cfg = Config::load(dir.path());
    // Should fall back to defaults, not crash
    assert!(cfg.disabled_rules.is_empty());
    assert_eq!(cfg.thresholds.file_lines_error, 500);
}
