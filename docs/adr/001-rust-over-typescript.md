# ADR-001: Rust over TypeScript for CLI

## Decision
Build claude-native in Rust, not TypeScript.

## Reason
- Single binary, no runtime — users don't need Node.js installed
- Fast startup (<50ms) — critical for --watch mode and pre-commit hooks
- Every competitor uses TypeScript — Rust differentiates us
- Correctness guarantees (no null pointer exceptions in rule logic)

## Alternatives Considered
- TypeScript: easier ecosystem (npx), but requires Node runtime
- Go: good binary story, but weaker ecosystem for CLI tools
