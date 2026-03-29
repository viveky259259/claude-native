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

// ═══════════════════════════════════════════════════════════════════
// R6: Circular dependency detection (highlight only)
// ═══════════════════════════════════════════════════════════════════

pub struct CircularDependencyCheck;

impl Rule for CircularDependencyCheck {
    fn id(&self) -> &str { "7.8" }
    fn name(&self) -> &str { "No circular dependencies detected" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        // Simple heuristic: check for mutual imports between file pairs
        let cycles = find_simple_cycles(ctx);
        if cycles.is_empty() {
            self.pass()
        } else {
            self.warn(
                &format!("{} potential circular dependency(ies) found", cycles.len()),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Review circular dependencies".into(),
                    description: format!(
                        "Circular imports force Claude to read files in loops. Found:\n{}\n\
                         Consider extracting shared types into a common module.",
                        cycles.iter().take(3).cloned().collect::<Vec<_>>().join("\n")
                    ),
                    effort: Effort::HalfDay,
                },
            )
        }
    }
}

fn find_simple_cycles(ctx: &ProjectContext) -> Vec<String> {
    let mut cycles = Vec::new();
    let source = ctx.source_files();
    let files: Vec<_> = source.iter()
        .filter(|f| f.line_count > 10)
        .take(30)
        .collect();

    for (i, a) in files.iter().enumerate() {
        let a_name = a.relative_path.file_stem()
            .and_then(|n| n.to_str()).unwrap_or("");
        let a_content = std::fs::read_to_string(&a.path).unwrap_or_default();

        for b in files.iter().skip(i + 1) {
            let b_name = b.relative_path.file_stem()
                .and_then(|n| n.to_str()).unwrap_or("");
            let b_content = std::fs::read_to_string(&b.path).unwrap_or_default();

            let a_imports_b = a_content.contains(b_name) && is_import_line(&a_content, b_name);
            let b_imports_a = b_content.contains(a_name) && is_import_line(&b_content, a_name);

            if a_imports_b && b_imports_a {
                cycles.push(format!(
                    "  {} <-> {}", a.relative_path.display(), b.relative_path.display()
                ));
            }
        }
    }
    cycles
}

fn is_import_line(content: &str, module: &str) -> bool {
    content.lines().any(|l| {
        let t = l.trim();
        (t.starts_with("use ") || t.starts_with("import ")
            || t.starts_with("from ") || t.starts_with("require("))
            && t.contains(module)
    })
}

// ═══════════════════════════════════════════════════════════════════
// R7: Co-location suggestion with cross-references
// ═══════════════════════════════════════════════════════════════════

pub struct ScatteredCodeCrossRefs;

impl Rule for ScatteredCodeCrossRefs {
    fn id(&self) -> &str { "7.9" }
    fn name(&self) -> &str { "Scattered code has cross-references" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        // Detect scattered patterns: same prefix in multiple directories
        let scattered = find_scattered_concepts(ctx);
        if scattered.is_empty() {
            return self.pass();
        }

        // Check if folder CLAUDE.md files mention cross-references
        let has_cross_refs = ctx.subdirectory_claude_mds.iter().any(|p| {
            std::fs::read_to_string(p).unwrap_or_default().contains("also in")
                || std::fs::read_to_string(p).unwrap_or_default().contains("related:")
                || std::fs::read_to_string(p).unwrap_or_default().contains("see also")
        });

        if has_cross_refs {
            self.pass()
        } else {
            self.warn(
                &format!("Scattered concepts found: {}", scattered.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add cross-references for scattered code".into(),
                    description: format!(
                        "These concepts are split across directories: {}.\n\
                         If co-location isn't possible, add cross-references in folder CLAUDE.md files:\n\
                         `# handlers/\\nUser logic also in: validators/user.rs, serializers/user.rs`\n\
                         This saves Claude ~300 tokens per scattered concept lookup.",
                        scattered.join(", ")
                    ),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

fn find_scattered_concepts(ctx: &ProjectContext) -> Vec<String> {
    use std::collections::{HashMap, HashSet};

    let mut concept_dirs: HashMap<String, HashSet<String>> = HashMap::new();
    for f in ctx.source_files() {
        let stem = f.path.file_stem().and_then(|n| n.to_str()).unwrap_or("");
        let dir = f.path.parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Extract concept prefix: user_handler → user, auth_service → auth
        let concept = stem.split('_').next().unwrap_or(stem);
        if concept.len() < 3 || concept == "mod" || concept == "index" || concept == "lib" {
            continue;
        }
        concept_dirs.entry(concept.to_string())
            .or_default()
            .insert(dir.to_string());
    }

    concept_dirs.into_iter()
        .filter(|(_, dirs)| dirs.len() >= 3) // same prefix in 3+ directories
        .map(|(concept, _)| concept)
        .take(5)
        .collect()
}

// R10 (MixedGeneratedCode) and R13 (MemoryDirectoryExists) are in token_checks.rs
