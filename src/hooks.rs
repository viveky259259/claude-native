use std::fs;
use std::path::Path;

use anyhow::Result;
use owo_colors::OwoColorize;

// ── Public API ──────────────────────────────────────────────────────

/// Generate and install a pre-commit hook that checks claude-native score.
pub fn install_hook(root: &Path, min_score: u32) -> Result<()> {
    let hooks_dir = root.join(".git").join("hooks");
    if !hooks_dir.exists() {
        anyhow::bail!("Not a git repository (no .git/hooks). Run `git init` first.");
    }

    let hook_path = hooks_dir.join("pre-commit");
    let script = generate_hook_script(min_score);
    fs::write(&hook_path, script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))?;
    }

    println!("{}", "Pre-commit hook installed!".green().bold());
    println!("  Path: {}", hook_path.display());
    println!("  Min score: {min_score}");
    println!();
    println!("Commits will be blocked if claude-native score drops below {min_score}.");
    Ok(())
}

/// Print the pre-commit config for .pre-commit-config.yaml integration.
pub fn print_precommit_config() {
    println!("Add to .pre-commit-config.yaml:");
    println!();
    println!("  - repo: local");
    println!("    hooks:");
    println!("      - id: claude-native");
    println!("        name: Claude Native Score Check");
    println!("        entry: claude-native");
    println!("        language: system");
    println!("        pass_filenames: false");
    println!("        always_run: true");
}

// ── Private helpers ─────────────────────────────────────────────────

fn generate_hook_script(min_score: u32) -> String {
    format!(
        r#"#!/bin/sh
# claude-native pre-commit hook
# Blocks commits if score drops below {min_score}

if ! command -v claude-native &> /dev/null; then
    echo "claude-native not found. Install: cargo install claude-native"
    exit 0
fi

SCORE=$(claude-native -o json 2>/dev/null | grep '"score"' | head -1 | sed 's/[^0-9.]//g' | cut -d. -f1)

if [ -z "$SCORE" ]; then
    echo "claude-native: could not determine score, skipping check"
    exit 0
fi

if [ "$SCORE" -lt {min_score} ]; then
    echo ""
    echo "claude-native: score $SCORE is below minimum {min_score}"
    echo "Run 'claude-native' to see suggestions, or 'claude-native --fix' to auto-repair."
    echo ""
    exit 1
fi

echo "claude-native: score $SCORE (>= {min_score}) OK"
exit 0
"#
    )
}
