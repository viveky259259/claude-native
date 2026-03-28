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
