# claude-native — Pending Tasks

## v1.0 Must-Have

- [x] **T1: Suggest `--init` when score < 60** — When overall score is below C grade, the #1 suggestion should be: "Run `claude-native --init` to bootstrap your project." (~10 min)
- [x] **T2: Update GOLDEN_RULES.md** — Function threshold 50→80, add lessons learned from self-scoring exercise (~30 min)
- [x] **T3: Publish to crates.io** (Cargo.toml ready, needs `git init && cargo publish`) — Add repository, homepage, keywords to Cargo.toml. `cargo publish` (~30 min)

## v1.0 Should-Have

- [x] **T4: Suggestions show exact file content** — Each suggestion for a missing file should include the exact content `--init` would generate, tailored to detected project type. Not just "create .claudeignore" but the actual patterns. (~2 hours)
- [x] **T5: Global .claudeignore filter** — Add `source_files()` method to ProjectContext that excludes claudeignored files. All rules should use it instead of raw `all_files`. (~1 hour)
- [x] **T6: Count test functions, not just test files** — Rule μ2 should count `#[test]`, `test(`, `def test_` occurrences, not just file count. A file with 30 tests should count more than a file with 1. (~1 hour)

## v1.1 Nice-to-Have

- [x] **T7: `--fix` mode** — Auto-apply all Quick Win suggestions: generate missing files, add patterns to .claudeignore. Only creates/appends, never overwrites. (~3 hours)
- [x] **T8: Token-cost estimates in suggestions** — Add estimated savings: "Create .claudeignore — saves ~2000 tokens per Glob/Grep", "Add Cargo.lock to .claudeignore — 941 lines = ~1900 tokens per read". (~2 hours)
- [x] **T9: `--diff` mode** — Show before/after score comparison. Run scan, apply --init, re-scan, show delta per dimension. (~1 hour)
- [x] **T10: `--watch` mode** — Re-score on file changes using filesystem watcher. Live score in terminal. (~2 hours)

## v2.0 — Next Release

- [ ] **T16: Multi-repo / distributed repo support** — Scan multiple repos at once or scan a parent directory containing multiple repos. Aggregate scores, detect cross-repo dependencies, generate per-repo + combined scorecards. Support `--repos dir1,dir2,dir3` flag and `claude-native --scan-all /path/to/workspace`. Handle monorepo-of-repos patterns (Git submodules, repo manifests).

- [ ] **T17: Report generation** — Generate structured reports in multiple formats: HTML report with charts/graphs, PDF export, Markdown summary for PRs/docs, JSON for dashboards. Include: score trend over time, per-dimension breakdown, top suggestions with effort estimates, comparison across repos (for multi-repo). Support `--report html` and `--report pdf` flags.

- [ ] **T18: Automation using OpenAI Codex / AI agents** — Enable automated fixing beyond quick wins using AI agents. Connect to OpenAI Codex or Claude API to: auto-generate CLAUDE.md content by reading the actual codebase, auto-split mega-files by analyzing function boundaries, auto-add type annotations to untyped code, auto-generate folder CLAUDE.md by reading folder contents. Support `--auto-fix` flag with `--ai-provider codex|claude` option. Requires API key via env var or `.claude-native.yml` config.
