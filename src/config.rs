use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;

/// User configuration from .claude-native.yml
#[derive(Debug, Default, Deserialize)]
pub struct Config {
    /// Rules to disable by ID (e.g., ["4.7", "μ2"])
    #[serde(default)]
    pub disabled_rules: Vec<String>,

    /// Override the auto-detected project type
    #[serde(default)]
    pub project_type: Option<String>,

    /// Custom thresholds
    #[serde(default)]
    pub thresholds: Thresholds,
}

#[derive(Debug, Deserialize)]
pub struct Thresholds {
    /// Max lines per file before warning (default: 300) and error (default: 500)
    #[serde(default = "default_file_warn")]
    pub file_lines_warn: usize,
    #[serde(default = "default_file_error")]
    pub file_lines_error: usize,

    /// Max lines per function before warning (default: 50) and error (default: 80)
    #[serde(default = "default_fn_warn")]
    pub function_lines_warn: usize,
    #[serde(default = "default_fn_error")]
    pub function_lines_error: usize,

    /// Max lines for CLAUDE.md (default: 200)
    #[serde(default = "default_claude_md_max")]
    pub claude_md_max_lines: usize,
}

fn default_file_warn() -> usize { 300 }
fn default_file_error() -> usize { 500 }
fn default_fn_warn() -> usize { 50 }
fn default_fn_error() -> usize { 80 }
fn default_claude_md_max() -> usize { 200 }

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            file_lines_warn: 300,
            file_lines_error: 500,
            function_lines_warn: 50,
            function_lines_error: 80,
            claude_md_max_lines: 200,
        }
    }
}

impl Config {
    /// Load config from .claude-native.yml in the project root.
    /// Returns default config if file doesn't exist.
    pub fn load(root: &Path) -> Self {
        let path = root.join(".claude-native.yml");
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_yaml::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn is_rule_disabled(&self, rule_id: &str) -> bool {
        self.disabled_rules.iter().any(|r| r == rule_id)
    }

    pub fn disabled_set(&self) -> HashSet<String> {
        self.disabled_rules.iter().cloned().collect()
    }
}
