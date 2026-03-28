use crate::detection::{MobileFramework, PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_mobile(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::Mobile(_))
}

// ── Rule MOB1: Platform build directories are ignored ───────────────

pub struct PlatformBuildDirsIgnored;

impl Rule for PlatformBuildDirsIgnored {
    fn id(&self) -> &str { "MOB1" }
    fn name(&self) -> &str { "Platform build dirs are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_mobile(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.claudeignore_content.is_none() {
            return self.fail(
                "No .claudeignore — mobile platform build dirs (50K+ files) are visible to Claude",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .claudeignore for mobile project".into(),
                    description: "Mobile projects generate massive build artifacts. Add: build/, .dart_tool/, ios/Pods/, android/.gradle/, android/build/, .expo/, DerivedData/, node_modules/".into(),
                    effort: Effort::Minutes,
                },
            );
        }

        let pt = ctx.project_type.as_ref().unwrap();
        let required: Vec<&str> = match &pt.primary {
            PrimaryType::Mobile(MobileFramework::Flutter) =>
                vec!["build", ".dart_tool", "Pods", ".gradle"],
            PrimaryType::Mobile(MobileFramework::ReactNative) =>
                vec!["node_modules", "Pods", ".gradle", ".expo"],
            PrimaryType::Mobile(MobileFramework::IosNative) =>
                vec!["Pods", "DerivedData", ".build"],
            PrimaryType::Mobile(MobileFramework::AndroidNative) =>
                vec![".gradle", "build", "intermediates"],
            _ => vec![],
        };

        let missing: Vec<&&str> = required.iter()
            .filter(|p| !ctx.claudeignore_contains(p))
            .collect();

        if missing.is_empty() {
            self.pass()
        } else {
            self.fail(
                &format!("Platform dirs not in .claudeignore: {}", missing.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add platform dirs to .claudeignore".into(),
                    description: format!("Add these to .claudeignore: {}", missing.iter().map(|m| format!("{}/", m)).collect::<Vec<_>>().join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule MOB2: Generated code is ignored ────────────────────────────

pub struct GeneratedCodeIgnored;

impl Rule for GeneratedCodeIgnored {
    fn id(&self) -> &str { "MOB2" }
    fn name(&self) -> &str { "Generated code is in .claudeignore" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_mobile(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let pt = ctx.project_type.as_ref().unwrap();
        let patterns: Vec<&str> = match &pt.primary {
            PrimaryType::Mobile(MobileFramework::Flutter) =>
                vec!["*.g.dart", "*.freezed.dart", "*.pb.dart"],
            PrimaryType::Mobile(MobileFramework::ReactNative) =>
                vec!["*.pb.js", ".bundle"],
            PrimaryType::Mobile(MobileFramework::IosNative) =>
                vec!["*.generated.swift", "*.pb.swift"],
            PrimaryType::Mobile(MobileFramework::AndroidNative) =>
                vec!["BuildConfig", "R.java", "R.kt", "databinding"],
            _ => vec![],
        };

        if patterns.is_empty() {
            return self.pass();
        }

        let gen_count = ctx.all_files.iter().filter(|f| f.is_generated).count();
        if gen_count == 0 {
            return self.pass();
        }

        let has_gen_patterns = patterns.iter().any(|p| ctx.claudeignore_contains(p));
        if has_gen_patterns {
            self.pass()
        } else {
            self.warn(
                &format!("{gen_count} generated files found — consider adding codegen patterns to .claudeignore"),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore generated mobile code".into(),
                    description: format!("Add these patterns to .claudeignore: {}", patterns.join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule MOB3: Platform-specific build/test commands ────────────────

pub struct PlatformCommands;

impl Rule for PlatformCommands {
    fn id(&self) -> &str { "MOB3" }
    fn name(&self) -> &str { "Platform-specific commands documented" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_mobile(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let pt = ctx.project_type.as_ref().unwrap();
        let expected_cmds: Vec<&str> = match &pt.primary {
            PrimaryType::Mobile(MobileFramework::Flutter) =>
                vec!["flutter analyze", "flutter test"],
            PrimaryType::Mobile(MobileFramework::ReactNative) =>
                vec!["npm test", "npx expo"],
            PrimaryType::Mobile(MobileFramework::IosNative) =>
                vec!["swift build", "swift test", "xcodebuild"],
            PrimaryType::Mobile(MobileFramework::AndroidNative) =>
                vec!["gradlew", "gradle"],
            _ => vec![],
        };

        let found = expected_cmds.iter().any(|cmd| content.contains(cmd));
        if found {
            self.pass()
        } else {
            self.fail(
                "CLAUDE.md lacks platform-specific build/test commands",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add platform-specific commands".into(),
                    description: format!("Add to CLAUDE.md: {}", expected_cmds.join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule MOB4: Binary assets excluded from search ───────────────────

pub struct BinaryAssetsExcluded;

impl Rule for BinaryAssetsExcluded {
    fn id(&self) -> &str { "MOB4" }
    fn name(&self) -> &str { "Binary assets excluded from search" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_mobile(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let asset_patterns = ["*.png", "*.jpg", "*.svg", "*.ttf", "*.mp3", "*.wav"];
        let has_assets_ignored = ctx.claudeignore_content.as_ref().map(|c| {
            asset_patterns.iter().any(|p| c.contains(p))
        }).unwrap_or(false);

        let has_assets_dir = ctx.root.join("assets").is_dir()
            || ctx.root.join("Assets").is_dir()
            || ctx.root.join("res").is_dir();

        if !has_assets_dir {
            return self.pass();
        }

        if has_assets_ignored {
            self.pass()
        } else {
            self.warn(
                "Binary assets (images, fonts, audio) may clutter Claude's searches",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Exclude binary assets from .claudeignore".into(),
                    description: "Add to .claudeignore: *.png, *.jpg, *.svg, *.ttf, *.otf, *.mp3, *.wav. Binary files can't be read but slow down Glob/Grep searches.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule MOB5: Lightweight verification ─────────────────────────────

pub struct LightweightVerification;

impl Rule for LightweightVerification {
    fn id(&self) -> &str { "MOB5" }
    fn name(&self) -> &str { "Fast verification prioritized over full builds" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_mobile(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_fast_cmd = content.contains("analyze")
            || content.contains("lint")
            || content.contains("type-check")
            || content.contains("typecheck")
            || content.contains("--no-emit");

        if has_fast_cmd {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't mention fast verification (analyze/lint/type-check)",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Prioritize fast checks in CLAUDE.md".into(),
                    description: "Add fast verification commands before full builds: `flutter analyze` (seconds) before `flutter build` (minutes). Claude should use the fast path for quick feedback.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
