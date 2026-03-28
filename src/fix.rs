use std::fs;

use anyhow::Result;

use crate::init;
use crate::rules::RuleResult;
use crate::scan::ProjectContext;

/// Auto-fix quick wins: create missing files, append missing .claudeignore patterns.
/// Returns list of actions taken.
pub fn apply_fixes(ctx: &ProjectContext, results: &[RuleResult]) -> Result<Vec<String>> {
    let mut actions = Vec::new();

    // Fix missing foundation files via --init
    let init_actions = init::init_project(ctx)?;
    for a in &init_actions {
        actions.push(format!("Created {a}"));
    }

    // Fix missing .claudeignore patterns
    let ignore_actions = fix_claudeignore(ctx, results)?;
    actions.extend(ignore_actions);

    // Fix missing .claude subdirectories
    fix_claude_dirs(ctx, &mut actions)?;

    Ok(actions)
}

fn fix_claudeignore(ctx: &ProjectContext, results: &[RuleResult]) -> Result<Vec<String>> {
    let mut actions = Vec::new();
    let ignore_path = ctx.root.join(".claudeignore");

    // Collect patterns that rules say are missing
    let mut missing_patterns: Vec<&str> = Vec::new();

    for r in results {
        if !r.status.is_failure() && !r.status.is_warning() { continue; }
        match r.rule_id.as_str() {
            "2.3" => missing_patterns.extend_from_slice(&[
                "Cargo.lock", "package-lock.json", "yarn.lock", "go.sum",
            ]),
            "2.4" => missing_patterns.extend_from_slice(&[
                "dist/", "build/", "target/", "coverage/",
            ]),
            _ => {}
        }
    }

    if missing_patterns.is_empty() { return Ok(actions); }

    let existing = fs::read_to_string(&ignore_path).unwrap_or_default();
    let mut to_add: Vec<&str> = missing_patterns.iter()
        .filter(|p| !existing.contains(*p))
        .copied()
        .collect();
    to_add.dedup();

    if !to_add.is_empty() {
        let mut content = existing;
        if !content.ends_with('\n') && !content.is_empty() {
            content.push('\n');
        }
        content.push_str("\n# Auto-added by claude-native --fix\n");
        for p in &to_add {
            content.push_str(p);
            content.push('\n');
        }
        fs::write(&ignore_path, content)?;
        actions.push(format!("Appended {} patterns to .claudeignore", to_add.len()));
    }

    Ok(actions)
}

fn fix_claude_dirs(ctx: &ProjectContext, actions: &mut Vec<String>) -> Result<()> {
    let rules_dir = ctx.root.join(".claude").join("rules");
    if !rules_dir.exists() {
        fs::create_dir_all(&rules_dir)?;
        fs::write(rules_dir.join(".gitkeep"), "")?;
        actions.push("Created .claude/rules/".into());
    }

    let skills_dir = ctx.root.join(".claude").join("skills");
    if !skills_dir.exists() {
        fs::create_dir_all(&skills_dir)?;
        fs::write(skills_dir.join(".gitkeep"), "")?;
        actions.push("Created .claude/skills/".into());
    }

    let mcp_path = ctx.root.join(".claude").join(".mcp.json");
    if !mcp_path.exists() {
        fs::write(&mcp_path, "{\"mcpServers\": {}}\n")?;
        actions.push("Created .claude/.mcp.json".into());
    }

    Ok(())
}
