use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_ml(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::ML)
}

// ── Rule DS1: Notebooks are not primary source ──────────────────────

pub struct NotebooksNotPrimary;

impl Rule for NotebooksNotPrimary {
    fn id(&self) -> &str { "DS1" }
    fn name(&self) -> &str { "Core logic in .py files, not notebooks" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_ml(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let notebook_count = ctx.all_files.iter().filter(|f| {
            f.path.extension().map(|e| e == "ipynb").unwrap_or(false)
        }).count();
        let py_count = ctx.all_files.iter().filter(|f| {
            f.path.extension().map(|e| e == "py").unwrap_or(false) && !f.is_test
        }).count();

        if notebook_count == 0 {
            return self.pass();
        }

        if py_count >= notebook_count {
            self.pass()
        } else {
            self.fail(
                &format!("{notebook_count} notebooks vs {py_count} .py files — notebooks are too dominant"),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Extract core logic from notebooks to .py".into(),
                    description: "Notebooks with outputs cost 10-50x more tokens than equivalent .py files. Extract training loops, data processing, and model definitions into .py files.".into(),
                    effort: Effort::HalfDay,
                },
            )
        }
    }
}

// ── Rule DS2: Model files ignored ───────────────────────────────────

pub struct ModelFilesIgnored;

impl Rule for ModelFilesIgnored {
    fn id(&self) -> &str { "DS2" }
    fn name(&self) -> &str { "Model weight files are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_ml(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let model_exts = ["pkl", "h5", "pth", "onnx", "safetensors", "bin", "pt"];
        let has_models = ctx.all_files.iter().any(|f| {
            f.path.extension().and_then(|e| e.to_str())
                .map(|e| model_exts.contains(&e))
                .unwrap_or(false)
        });

        if !has_models {
            return self.pass();
        }

        let ignored = model_exts.iter().any(|e| ctx.claudeignore_contains(e));
        if ignored {
            self.pass()
        } else {
            self.fail(
                "Model weight files found but not in .claudeignore",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore model files".into(),
                    description: "Add *.pkl, *.h5, *.pth, *.onnx, *.safetensors, *.bin to .claudeignore. Model files are binary (100MB-10GB) — Claude can't read them.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule DS3: Dataset files ignored ─────────────────────────────────

pub struct DatasetFilesIgnored;

impl Rule for DatasetFilesIgnored {
    fn id(&self) -> &str { "DS3" }
    fn name(&self) -> &str { "Dataset files are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_ml(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let data_dir = ctx.root.join("data");
        let has_data = data_dir.is_dir() || ctx.all_files.iter().any(|f| {
            f.path.extension().and_then(|e| e.to_str())
                .map(|e| matches!(e, "csv" | "parquet" | "feather" | "arrow"))
                .unwrap_or(false)
                && f.size_bytes > 1_000_000
        });

        if !has_data {
            return self.pass();
        }

        let ignored = ctx.claudeignore_contains("data/")
            || ctx.claudeignore_contains("*.csv")
            || ctx.claudeignore_contains("*.parquet");

        if ignored {
            self.pass()
        } else {
            self.fail(
                "Dataset files/directories found but not in .claudeignore",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore dataset files".into(),
                    description: "Add data/, *.csv, *.parquet, *.feather to .claudeignore. Claude should sample data via `head -20 data.csv`, not read entire files.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule DS4: Experiment workflow documented ─────────────────────────

pub struct ExperimentWorkflowDocumented;

impl Rule for ExperimentWorkflowDocumented {
    fn id(&self) -> &str { "DS4" }
    fn name(&self) -> &str { "Experiment workflow documented" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_ml(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_workflow = content.contains("train")
            || content.contains("evaluat")
            || content.contains("experiment")
            || content.contains("metric");

        if has_workflow {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't document the ML experiment workflow",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Document experiment workflow".into(),
                    description: "Add to CLAUDE.md: how to run training, evaluate, what metrics matter, where results are stored. Without this, Claude runs the wrong step.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule DS5: Requirements pin exact versions ───────────────────────

pub struct RequirementsPinned;

impl Rule for RequirementsPinned {
    fn id(&self) -> &str { "DS5" }
    fn name(&self) -> &str { "Requirements pin exact versions" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_ml(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match ctx.read_root_file("requirements.txt") {
            Some(c) => c,
            None => return self.pass(),
        };

        let total_deps = content.lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#') && !l.starts_with('-'))
            .count();
        let pinned = content.lines()
            .filter(|l| l.contains("=="))
            .count();

        if total_deps == 0 {
            return self.pass();
        }

        let ratio = pinned as f64 / total_deps as f64;
        if ratio >= 0.8 {
            self.pass()
        } else {
            self.warn(
                &format!("Only {:.0}% of dependencies use exact versions (==)", ratio * 100.0),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Pin ML dependency versions".into(),
                    description: "ML deps have complex compatibility (CUDA, framework versions). Use == pinning: `torch==2.1.0` not `torch>=2.0`. Prevents 'works on my machine' failures.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
