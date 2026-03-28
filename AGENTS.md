# claude-native

> Universal AI agent instructions (AGENTS.md standard)

## Build & Test

- Build: `cargo build`
- Test: `cargo test`
- Run: `cargo run -- /path/to/project`

## Guidelines

- Every rule implements the `Rule` trait in `src/rules/mod.rs`
- Use `self.pass()`, `self.fail()`, `self.warn()`, `self.skip()` helpers
- Project-specific rules override `applies_to()` to filter by `ProjectType`
- Keep functions under 80 lines
- Add tests for new rules in `tests/`
- Rule IDs must match GOLDEN_RULES.md spec
