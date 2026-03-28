pub mod builder;
pub mod classifiers;
pub mod file_stats;

use std::path::PathBuf;

use globset::GlobSet;

pub use builder::build_context;

use crate::detection::ProjectType;

/// A single file's metadata
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub line_count: usize,
    pub size_bytes: u64,
    pub is_test: bool,
    pub is_generated: bool,
}

/// A discovered package manifest
#[derive(Debug, Clone)]
pub struct ManifestInfo {
    pub path: PathBuf,
    pub kind: ManifestKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestKind {
    PackageJson,
    CargoToml,
    GoMod,
    PubspecYaml,
    RequirementsTxt,
    PyprojectToml,
    Gemfile,
    MixExs,
    Other(String),
}

/// Everything the rule engine needs about the project.
/// Built once during scan, shared immutably with all rules.
#[derive(Debug)]
pub struct ProjectContext {
    pub root: PathBuf,
    pub project_type: Option<ProjectType>,

    // File system state
    pub all_files: Vec<FileInfo>,
    pub total_file_count: usize,
    pub max_depth: usize,
    pub directories: Vec<PathBuf>,

    // Key file contents (read once, shared by many rules)
    pub claude_md_content: Option<String>,
    pub claude_md_path: Option<PathBuf>,
    pub claudeignore_content: Option<String>,
    pub readme_content: Option<String>,
    pub settings_json: Option<serde_json::Value>,
    pub package_json: Option<serde_json::Value>,
    pub package_manifests: Vec<ManifestInfo>,

    pub agents_md_content: Option<String>,

    // Derived analysis
    pub has_claude_dir: bool,
    pub has_claude_rules_dir: bool,
    pub has_claude_skills_dir: bool,
    pub has_claude_agents_dir: bool,
    pub subdirectory_claude_mds: Vec<PathBuf>,
    pub test_files: Vec<PathBuf>,
    pub ci_configs: Vec<PathBuf>,
    pub env_files: Vec<PathBuf>,
    pub lock_files: Vec<PathBuf>,
    pub mcp_json_path: Option<PathBuf>,

    // Compiled .claudeignore glob set
    pub(crate) ignore_set: Option<GlobSet>,
    // Raw patterns from .claudeignore (for claudeignore_contains checks)
    pub(crate) ignore_patterns: Vec<String>,

    // Cached file reads
    root_file_cache: std::collections::HashMap<String, String>,
}

impl ProjectContext {
    pub fn has_file(&self, relative: &str) -> bool {
        self.root.join(relative).exists()
    }

    pub fn has_claude_md(&self) -> bool {
        self.claude_md_content.is_some()
    }

    pub fn read_root_file(&self, relative: &str) -> Option<&str> {
        self.root_file_cache.get(relative).map(|s| s.as_str())
    }

    pub fn read_manifest_content(&self, filename: &str) -> Option<&str> {
        self.root_file_cache.get(filename).map(|s| s.as_str())
    }

    /// Get source files excluding tests, generated, and claudeignored files.
    pub fn source_files(&self) -> Vec<&FileInfo> {
        self.all_files.iter()
            .filter(|f| !f.is_test && !f.is_generated
                && !self.is_claudeignored(&f.relative_path.to_string_lossy()))
            .collect()
    }

    pub fn source_file_count(&self) -> usize {
        self.source_files().len()
    }

    pub fn average_source_file_lines(&self) -> f64 {
        let files: Vec<_> = self.source_files().into_iter()
            .filter(|f| f.line_count > 0)
            .collect();
        if files.is_empty() { return 0.0; }
        let total: usize = files.iter().map(|f| f.line_count).sum();
        total as f64 / files.len() as f64
    }

    /// Count actual test functions across all test files.
    pub fn test_function_count(&self) -> usize {
        let test_markers = ["#[test]", "fn test", "test(", "test '", "test \"", "it(", "it '", "def test_", "async fn test"];
        self.test_files.iter().map(|tf| {
            std::fs::read_to_string(tf).unwrap_or_default()
                .lines()
                .filter(|l| test_markers.iter().any(|m| l.trim().starts_with(m)))
                .count()
        }).sum()
    }

    pub fn claude_md_line_count(&self) -> usize {
        self.claude_md_content
            .as_ref()
            .map(|c| c.lines().count())
            .unwrap_or(0)
    }

    pub fn readme_line_count(&self) -> usize {
        self.readme_content
            .as_ref()
            .map(|c| c.lines().count())
            .unwrap_or(0)
    }

    /// Check if .claudeignore contains a pattern that would match the given string.
    /// Used by rules to verify specific things are being ignored.
    pub fn claudeignore_contains(&self, pattern: &str) -> bool {
        self.ignore_patterns.iter().any(|p| {
            p.contains(pattern) || pattern.contains(p.trim_end_matches('/'))
        })
    }

    pub fn settings_has_permissions(&self) -> bool {
        self.settings_json
            .as_ref()
            .and_then(|v| v.get("permissions"))
            .and_then(|p| p.get("allow"))
            .and_then(|a| a.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    }

    pub fn settings_has_hooks(&self) -> bool {
        self.settings_json
            .as_ref()
            .and_then(|v| v.get("hooks"))
            .map(|h| h.is_object() && h.as_object().map(|o| !o.is_empty()).unwrap_or(false))
            .unwrap_or(false)
    }

    pub fn has_post_tool_use_hook_for_format(&self) -> bool {
        self.settings_json
            .as_ref()
            .and_then(|v| v.get("hooks"))
            .and_then(|h| h.get("PostToolUse"))
            .map(|ptu| {
                let s = serde_json::to_string(ptu).unwrap_or_default();
                s.contains("Edit") || s.contains("Write")
            })
            .unwrap_or(false)
    }

    pub fn has_pre_tool_use_protection_hook(&self) -> bool {
        self.settings_json
            .as_ref()
            .and_then(|v| v.get("hooks"))
            .and_then(|h| h.get("PreToolUse"))
            .is_some()
    }

    /// Check if a file path matches any .claudeignore glob pattern.
    pub fn is_claudeignored(&self, relative_path: &str) -> bool {
        if let Some(ref gs) = self.ignore_set {
            gs.is_match(relative_path)
        } else {
            false
        }
    }

    pub fn mega_files(&self, threshold: usize) -> Vec<&FileInfo> {
        self.all_files.iter()
            .filter(|f| {
                f.line_count > threshold
                    && !self.is_claudeignored(&f.relative_path.to_string_lossy())
            })
            .collect()
    }

    pub fn workspace_packages(&self) -> Vec<&PathBuf> {
        let workspace_dirs = ["packages", "apps", "services", "libs"];
        self.directories.iter()
            .filter(|d| {
                if let Ok(rel) = d.strip_prefix(&self.root) {
                    let components: Vec<_> = rel.components().collect();
                    components.len() == 1 && workspace_dirs.iter().any(|wd| {
                        d.parent().map(|p| p.file_name().map(|n| n == *wd).unwrap_or(false)).unwrap_or(false)
                    })
                } else {
                    false
                }
            })
            .collect()
    }
}

