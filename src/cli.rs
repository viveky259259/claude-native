use std::path::PathBuf;

use clap::Parser;

/// Scan a project and score how Claude Native it is.
#[derive(Parser, Debug)]
#[command(name = "claude-native", version, about)]
pub struct Cli {
    /// Path to the project directory to scan (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Only show failures and suggestions (hide passing rules)
    #[arg(short, long)]
    pub failures_only: bool,

    /// Output format
    #[arg(short = 'o', long, default_value = "terminal")]
    pub format: OutputFormat,

    /// Bootstrap the project: generate CLAUDE.md, .claudeignore, .claude/settings.json
    #[arg(long)]
    pub init: bool,

    /// Auto-fix: apply all quick-win suggestions (create missing files, append ignore patterns)
    #[arg(long)]
    pub fix: bool,

    /// Show estimated score improvement without making changes
    #[arg(long)]
    pub diff: bool,

    /// Watch for file changes and re-score automatically
    #[arg(long)]
    pub watch: bool,

    /// Output a shields.io badge URL for your README
    #[arg(long)]
    pub badge: bool,

    /// Run as MCP server (JSON-RPC over stdio)
    #[arg(long)]
    pub mcp: bool,

    /// Install pre-commit hook (blocks commits below score threshold)
    #[arg(long, value_name = "MIN_SCORE")]
    pub hook: Option<u32>,

    /// Like --init but also generates .cursorrules and copilot-instructions.md
    #[arg(long)]
    pub init_all: bool,

    /// Record score to .claude-native-history.json and show trend
    #[arg(long)]
    pub history: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum OutputFormat {
    Terminal,
    Json,
}
