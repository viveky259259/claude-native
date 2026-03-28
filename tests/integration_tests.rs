mod helpers;

use claude_native::detection;
use claude_native::rules;
use claude_native::scoring;

/// Full end-to-end: scan → detect → rules → score for a well-configured project.
#[test]
fn well_configured_project_scores_high() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# My Project\nBuild: `cargo build`\nTest: `cargo test`\n## Patterns\nUse the repository pattern."),
        (".claudeignore", "target/\nCargo.lock\nnode_modules/\ndist/\nbuild/\ncoverage/\n*.log\n.env\n"),
        (".claude/settings.json", r#"{"permissions": {"allow": ["Bash(cargo test:*)"]},"hooks": {"PostToolUse": [{"matcher": "Edit|Write", "hooks": [{"type": "command", "command": "rustfmt"}]}], "PreToolUse": [{"matcher": "Read", "hooks": []}]}}"#),
        (".claude/rules/api.md", "---\npaths: src/api/**\n---\n# API rules"),
        (".claude/skills/deploy/SKILL.md", "# Deploy workflow"),
        (".claude/.mcp.json", r#"{"mcpServers": {}}"#),
        ("README.md", "# My Project\n\n## Quick Start\n\n```bash\ncargo run\n```\n"),
        ("Cargo.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\"\n"),
        ("Cargo.lock", "# lock content"),
        ("src/main.rs", "fn main() {\n    println!(\"hello\");\n}\n"),
        ("src/lib.rs", "pub mod handler;\npub mod utils;\n"),
        ("src/handler.rs", &"pub fn handle() -> String { \"ok\".into() }\n".repeat(5)),
        ("src/utils.rs", &"pub fn util() -> i32 { 42 }\n".repeat(5)),
        ("tests/handler_test.rs", "#[test]\nfn test_handle() {\n    assert_eq!(1, 1);\n}\n"),
        ("tests/utils_test.rs", "#[test]\nfn test_util() {\n    assert_eq!(42, 42);\n}\n"),
        (".github/workflows/ci.yml", "name: CI\non: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest"),
    ]);

    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let all = rules::all_rules();
    let results: Vec<_> = all.iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();

    let sc = scoring::calculate(results, &pt);

    assert!(sc.total_score >= 75.0, "Well-configured project should score ≥75, got {:.0}", sc.total_score);
    assert!(
        matches!(sc.grade, scoring::Grade::APlus | scoring::Grade::A | scoring::Grade::B),
        "Expected grade A+ or A or B, got {}",
        sc.grade
    );
}

/// Bare-bones project should score below B grade and have many suggestions.
#[test]
fn bare_project_scores_below_b() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
    ]);

    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let all = rules::all_rules();
    let results: Vec<_> = all.iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();

    let sc = scoring::calculate(results, &pt);

    assert!(sc.total_score < 70.0, "Bare project should score <70 (B), got {:.0}", sc.total_score);
    assert!(sc.suggestions.len() >= 5, "Should have many suggestions, got {}", sc.suggestions.len());
    // Foundation should be capped at 30 due to missing CLAUDE.md (CRITICAL)
    let foundation = sc.dimensions.iter().find(|d| d.dimension == rules::Dimension::Foundation).unwrap();
    assert!(foundation.capped, "Foundation should be capped without CLAUDE.md");
}

/// JSON output format produces valid JSON.
#[test]
fn json_output_is_valid() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test\nBuild: `make`\nTest: `make test`"),
        ("src/main.rs", "fn main() {}"),
        ("Cargo.toml", "[package]\nname = \"test\""),
    ]);

    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let all = rules::all_rules();
    let results: Vec<_> = all.iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();

    let sc = scoring::calculate(results, &pt);

    let json = serde_json::json!({
        "project_type": format!("{}", sc.project_type),
        "score": sc.total_score,
        "grade": format!("{}", sc.grade),
        "suggestions_count": sc.suggestions.len(),
    });

    let json_str = serde_json::to_string_pretty(&json).expect("JSON serialization failed");
    let parsed: serde_json::Value = serde_json::from_str(&json_str).expect("JSON parsing failed");
    assert!(parsed.get("score").is_some());
    assert!(parsed.get("grade").is_some());
}

/// Monorepo-specific rules only fire for monorepos.
#[test]
fn monorepo_rules_only_fire_for_monorepos() {
    // Standard project
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let all = rules::all_rules();
    let results: Vec<_> = all.iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();

    // No monorepo rule IDs should appear
    let monorepo_ids: Vec<_> = results.iter()
        .filter(|r| r.rule_id.starts_with('M') && r.rule_id.len() <= 2)
        .collect();
    assert!(monorepo_ids.is_empty(), "Monorepo rules should not fire for non-monorepo: {:?}", monorepo_ids.iter().map(|r| &r.rule_id).collect::<Vec<_>>());
}

/// Micro-repo rules fire for micro-repos.
#[test]
fn micro_repo_rules_fire_for_small_projects() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"tiny-lib\""),
        ("src/lib.rs", "pub fn hello() -> &'static str { \"hello\" }"),
    ]);
    let pt = detection::detect(&ctx);
    assert_eq!(pt.primary, claude_native::detection::PrimaryType::MicroRepo);
    ctx.project_type = Some(pt.clone());

    let all = rules::all_rules();
    let results: Vec<_> = all.iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();

    let micro_ids: Vec<_> = results.iter()
        .filter(|r| r.rule_id.starts_with('μ'))
        .collect();
    assert!(!micro_ids.is_empty(), "Micro-repo rules should fire for small projects");
}

/// Scan module correctly identifies test files.
#[test]
fn scan_identifies_test_files() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
        ("tests/main_test.rs", "#[test] fn it_works() {}"),
        ("src/__tests__/util.test.ts", "test('x', () => {})"),
        ("spec/helper_spec.rb", "describe 'helper' do; end"),
    ]);
    assert!(ctx.test_files.len() >= 3, "Expected ≥3 test files, found {}", ctx.test_files.len());
}

/// Scan identifies lock files.
#[test]
fn scan_identifies_lock_files() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.lock", ""),
        ("package-lock.json", "{}"),
    ]);
    assert_eq!(ctx.lock_files.len(), 2);
}

/// Scan identifies env files.
#[test]
fn scan_identifies_env_files() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".env", "SECRET=x"),
        (".env.local", "SECRET=y"),
        (".env.example", "SECRET="),
    ]);
    assert_eq!(ctx.env_files.len(), 3);
}

/// Scan calculates max depth correctly.
#[test]
fn scan_calculates_max_depth() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("a/b/c/d/e/deep.rs", "fn deep() {}"),
    ]);
    assert!(ctx.max_depth >= 5, "Expected depth ≥5, got {}", ctx.max_depth);
}
