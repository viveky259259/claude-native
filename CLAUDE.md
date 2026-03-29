# claude-native — Claude Native Score Scanner

Rust CLI that scans a project and scores how "Claude Native" it is.

## Build & Test

```
cargo build
cargo test
cargo test --test <name>        # Test single file
cargo test <module>::            # Test single module
cargo run -- /path/to/project
cargo run -- . -o json
```

## Architecture

Scan → Detect → Rules → Score → Output pipeline:
- `src/scan/` — walks directory, builds `ProjectContext` (file metadata, key file contents)
- `src/detection/` — 3-phase project type detection (structure → domain → micro-repo)
- `src/rules/` — 94 rules across 5 dimensions + 13 project-type modules
- `src/scoring/` — weighted score calculation, grade assignment (A+ through F)
- `src/output/` — colored terminal scorecard + JSON output

## Code Patterns

- Every rule implements the `Rule` trait (`src/rules/mod.rs`)
- Use `self.pass()`, `self.fail()`, `self.warn()`, `self.skip()` helpers — never construct `RuleResult` manually
- Project-specific rules override `applies_to()` to filter by `ProjectType`
- `ProjectContext` is built once, shared immutably with all rules
- Detection runs domain markers before micro-repo classification

## Adding a New Rule

1. Create struct in the appropriate rules file under `src/rules/`
2. Implement `Rule` trait (id, name, dimension, severity, check)
3. Register in `all_rules()` in `src/rules/mod.rs` or `project_specific_rules()` in `src/rules/project_specific/mod.rs`
4. Add test in `tests/`

## Conventions

- Snake_case for all Rust files
- Each rule file corresponds to one scoring dimension
- Suggestions must include priority (QuickWin/HighImpact/NiceToHave) and effort estimate
- GOLDEN_RULES.md is the spec — rule IDs in code must match the spec
