use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_game(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::GameDev(_))
}

// ── Rule GAME1: Binary scene/asset files ignored ────────────────────

pub struct BinaryAssetsIgnored;

impl Rule for BinaryAssetsIgnored {
    fn id(&self) -> &str { "GAME1" }
    fn name(&self) -> &str { "Binary scene/asset files are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_game(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let binary_patterns = [
            "*.unity", "*.prefab", "*.asset", "*.tscn", "*.tres",
            "*.blend", "*.fbx", "*.png", "*.wav", "*.mp3",
        ];

        if ctx.claudeignore_content.is_none() {
            return self.fail(
                "No .claudeignore — binary game assets (scenes, models, textures) clutter searches",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .claudeignore for game project".into(),
                    description: format!("Add to .claudeignore: {}\nClaude can't read binary files — they only waste search results.", binary_patterns.join(", ")),
                    effort: Effort::Minutes,
                },
            );
        }

        let has_patterns = binary_patterns.iter().any(|p| {
            let p = p.trim_start_matches("*.");
            ctx.claudeignore_contains(p)
        });

        if has_patterns {
            self.pass()
        } else {
            self.fail(
                "Binary game asset patterns not in .claudeignore",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore binary game assets".into(),
                    description: format!("Add to .claudeignore: {}", binary_patterns.join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule GAME2: Editor metadata ignored ─────────────────────────────

pub struct EditorMetadataIgnored;

impl Rule for EditorMetadataIgnored {
    fn id(&self) -> &str { "GAME2" }
    fn name(&self) -> &str { "Editor-generated metadata is ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_game(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let metadata = ["*.meta", "Library/", ".godot/", ".import/"];
        let relevant: Vec<&&str> = metadata.iter()
            .filter(|m| {
                let m_clean = m.trim_end_matches('/').trim_start_matches("*.");
                ctx.all_files.iter().any(|f| {
                    let path_str = f.path.to_string_lossy();
                    path_str.contains(m_clean)
                }) || ctx.root.join(m.trim_end_matches('/')).is_dir()
            })
            .collect();

        if relevant.is_empty() {
            return self.pass();
        }

        let all_ignored = relevant.iter().all(|m| {
            let m_clean = m.trim_end_matches('/').trim_start_matches("*.");
            ctx.claudeignore_contains(m_clean)
        });

        if all_ignored {
            self.pass()
        } else {
            self.fail(
                "Editor metadata not in .claudeignore",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore editor metadata".into(),
                    description: "Add *.meta, Library/, .godot/, .import/ to .claudeignore. Unity alone generates a .meta file per asset — 500 assets = 500 extra search results.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule GAME3: Scripts are the primary target ──────────────────────

pub struct ScriptsArePrimary;

impl Rule for ScriptsArePrimary {
    fn id(&self) -> &str { "GAME3" }
    fn name(&self) -> &str { "CLAUDE.md directs to script directories" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_game(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_script_ref = content.contains("script")
            || content.contains("src/")
            || content.contains("assets/scripts")
            || content.contains("shader");

        if has_script_ref {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't reference script directories",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Point CLAUDE.md to scripts".into(),
                    description: "Add to CLAUDE.md: 'Game logic: Assets/Scripts/ — Shaders: Assets/Shaders/'. Claude can only read code, not scenes or prefabs.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule GAME4: Tests exist for game logic ──────────────────────────

pub struct GameLogicTests;

impl Rule for GameLogicTests {
    fn id(&self) -> &str { "GAME4" }
    fn name(&self) -> &str { "Tests for non-visual game logic" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_game(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if !ctx.test_files.is_empty() {
            self.pass()
        } else {
            self.warn(
                "No tests for game logic (state machines, inventory, damage calculations)",
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Add tests for pure game logic".into(),
                    description: "Pure logic (math, state, rules) is perfectly testable. Add unit tests for game systems — Claude can verify changes without needing the game editor.".into(),
                    effort: Effort::HalfDay,
                },
            )
        }
    }
}
