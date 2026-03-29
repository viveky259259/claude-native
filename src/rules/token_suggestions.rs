use crate::detection::{Language, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

// ═══════════════════════════════════════════════════════════════════
// R4: Index/barrel file per module (language-aware)
// ═══════════════════════════════════════════════════════════════════

pub struct LanguageAwareIndexFiles;

impl Rule for LanguageAwareIndexFiles {
    fn id(&self) -> &str { "7.6" }
    fn name(&self) -> &str { "Modules have language-appropriate index files" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let pt = match &ctx.project_type {
            Some(pt) => pt,
            None => return self.skip(),
        };

        let expected = expected_index_files(&pt.languages);
        if expected.is_empty() { return self.pass(); }

        let source_dirs = significant_source_dirs(ctx);
        if source_dirs.is_empty() { return self.pass(); }

        let missing: Vec<String> = source_dirs.iter()
            .filter(|d| !expected.iter().any(|idx| d.join(idx).exists()))
            .filter_map(|d| d.strip_prefix(&ctx.root).ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        if missing.is_empty() {
            self.pass()
        } else {
            let idx_names = expected.join(" or ");
            self.warn(
                &format!("{} dirs lack {idx_names}", missing.len()),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: format!("Add {idx_names} to modules"),
                    description: format!(
                        "For your project ({:?}), add {idx_names} to each module directory. \
                         Claude reads the index file to understand module exports without scanning all files.\n\
                         Missing in: {}",
                        pt.languages.first().unwrap_or(&Language::Other("unknown".into())),
                        missing.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
                    ),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

fn expected_index_files(langs: &[Language]) -> Vec<&'static str> {
    for lang in langs {
        match lang {
            Language::Rust => return vec!["mod.rs"],
            Language::TypeScript => return vec!["index.ts", "index.tsx"],
            Language::JavaScript => return vec!["index.js", "index.jsx"],
            Language::Python => return vec!["__init__.py"],
            Language::Dart => return vec!["index.dart"],
            Language::Go => {}, // Go uses package-level, no index
            Language::Ruby => return vec!["index.rb"],
            Language::CSharp => {}, // Namespace-based
            _ => {}
        }
    }
    vec![]
}

fn significant_source_dirs(ctx: &ProjectContext) -> Vec<std::path::PathBuf> {
    use std::collections::HashMap;
    let skip = [".claude", ".github", "tests", "test", "docs", "examples", "target"];
    let mut counts: HashMap<std::path::PathBuf, usize> = HashMap::new();

    for f in ctx.source_files() {
        if let Some(parent) = f.path.parent() {
            if parent == ctx.root { continue; }
            let name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with('.') || skip.contains(&name) { continue; }
            *counts.entry(parent.to_path_buf()).or_insert(0) += 1;
        }
    }
    counts.into_iter().filter(|(_, c)| *c >= 3).map(|(p, _)| p).collect()
}

// ═══════════════════════════════════════════════════════════════════
// R5+R8: Public API and constants at top of files
// ═══════════════════════════════════════════════════════════════════

pub struct PublicApiAtTop;

impl Rule for PublicApiAtTop {
    fn id(&self) -> &str { "7.7" }
    fn name(&self) -> &str { "Public APIs/exports at top of files" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let mut buried_count = 0;
        let sample = ctx.source_files();
        let sample: Vec<_> = sample.iter().take(20).collect();

        for f in &sample {
            if f.line_count < 50 { continue; } // skip small files
            if let Ok(content) = std::fs::read_to_string(&f.path) {
                if has_buried_public_api(&content) {
                    buried_count += 1;
                }
            }
        }

        if buried_count == 0 {
            self.pass()
        } else {
            self.warn(
                &format!("{buried_count} files have public APIs below private code"),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Move public APIs to top of files".into(),
                    description: "Public functions/exports should be above private helpers. \
                        Claude reads top-down — if exports are at line 200, it reads 200 lines to find the API. \
                        Put `pub fn`, `export`, or constants at the top. Saves ~100 tokens per file read.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

fn has_buried_public_api(content: &str) -> bool {
    let mut first_private_line: Option<usize> = None;
    let mut last_public_line: Option<usize> = None;

    for (i, line) in content.lines().enumerate() {
        let t = line.trim();
        // Detect public markers
        if t.starts_with("pub fn ") || t.starts_with("pub struct ")
            || t.starts_with("pub enum ") || t.starts_with("pub type ")
            || t.starts_with("export ") || t.starts_with("export default")
        {
            last_public_line = Some(i);
        }
        // Detect private markers (non-pub function definitions)
        if (t.starts_with("fn ") && !t.starts_with("fn main"))
            || t.starts_with("struct ") || t.starts_with("enum ")
            || (t.starts_with("def ") && !t.starts_with("def _"))
            || t.starts_with("function ") || t.starts_with("const _")
        {
            if first_private_line.is_none() {
                first_private_line = Some(i);
            }
        }
    }

    // If last public line is significantly after first private line
    match (first_private_line, last_public_line) {
        (Some(priv_line), Some(pub_line)) => pub_line > priv_line + 20,
        _ => false,
    }
}

// R6, R7, R10, R13 are in token_checks.rs
