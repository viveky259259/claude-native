mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::legacy::*;

#[test]
fn dead_code_flagged_passes_when_documented() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Project\nBuild: `make`\nTest: `make test`\n# Dead code\n- deprecated/ is old code"),
        ("src/main.py", "print('hello')"),
        ("deprecated/old.py", "# old"),
    ]);
    let mut pt = claude_native::detection::detect(&ctx);
    pt.flags.is_legacy = true;
    ctx.project_type = Some(pt);
    let result = DeadCodeFlagged.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn correct_patterns_passes_with_reference() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Project\nBuild: `make`\nTest: `make test`\nFollow the pattern in src/services/auth.ts for error handling."),
        ("src/main.py", "print('hello')"),
    ]);
    let mut pt = claude_native::detection::detect(&ctx);
    pt.flags.is_legacy = true;
    ctx.project_type = Some(pt);
    let result = CorrectPatternsIdentified.check(&ctx);
    assert!(result.status.is_pass());
}
