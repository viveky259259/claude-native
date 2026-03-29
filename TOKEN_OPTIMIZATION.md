# Token Optimization: Beyond Current Rules

Ideas for reducing Claude's token consumption at each stage of its workflow.

## Stage 1: Context Loading (every request)

### What burns tokens:
- CLAUDE.md loads on EVERY request — every extra line costs tokens permanently
- Rules that match broad paths load unnecessarily
- Stale imports in CLAUDE.md (`@large-file.md`) inflate context

### New rules to add:
- **R1: CLAUDE.md has no duplicate content with README** — if CLAUDE.md copies README sections, Claude reads the same info twice. Check for >30% content overlap.
- **R2: .claude/rules/ files use narrow path: scopes** — a rule scoped to `**/*.rs` loads on every Rust file. Scoping to `src/api/**/*.rs` loads only when working on API code.

---

## Stage 2: Exploration (finding files)

### What burns tokens:
- Claude runs Glob → gets 50 results → reads 10 → finds the right one. Each wrong read = ~200 tokens wasted.
- No way to know what's in a folder without reading files inside it.

### New rules to add:
- **R3: Project has a MANIFEST.md or directory index** — a machine-readable map like:
  ```
  src/rules/ — Rule implementations (one file per dimension)
  src/scan/ — Directory scanner, builds ProjectContext
  src/detection/ — Project type detection (14 types)
  ```
  Claude reads this ONE file (~50 tokens) instead of running 5 Glob commands (~500 tokens).

- **R4: Entry point documented per module** — each major directory should declare its entry file. Claude starts there instead of guessing.

---

## Stage 3: Understanding (reading code)

### What burns tokens:
- Reading a 200-line file to find one 5-line function = 195 lines wasted
- No types → Claude reads function body to infer shapes
- Scattered related logic → Claude reads 5 files instead of 1

### New rules to add:
- **R5: Exports are at the top of files** — public API (exported functions, types) should be at the top. Claude reads top-down; if exports are at line 300, it reads 300 lines to find the API.

- **R6: No circular dependencies** — circular imports force Claude to read file A to understand B, then B to understand A. Detect import cycles and flag them.

- **R7: Related code is co-located** — if `User` struct, `UserService`, and `UserRepository` are in 3 different directories, Claude reads 3 files. If they're in `user/`, Claude reads 1 directory.

- **R8: Config/constants at top of file** — magic numbers buried in function bodies force Claude to read the entire file. Top-level constants are found in ~10 tokens.

---

## Stage 4: Editing (making changes)

### What burns tokens:
- Editing a long function → Claude must reproduce surrounding context in the Edit tool
- Editing a file with no clear boundaries → higher chance of wrong edit location
- Editing generated code that gets overwritten → entire edit wasted

### New rules to add:
- **R9: Functions have clear boundaries** — blank line before/after each function. Claude's Edit tool needs unique context strings; functions jammed together make edits ambiguous.

- **R10: No mixed generated + hand-written in same file** — if half the file is generated and half is handwritten, Claude may edit the wrong half. Split them.

---

## Stage 5: Verification (running tests/builds)

### What burns tokens:
- Running ALL tests after a small change → 5000-line output in context
- Verbose build output → CI-style logging wastes context
- No way to test a single file → must run entire suite

### New rules to add:
- **R11: Targeted test command documented** — CLAUDE.md should have: "Test single file: `cargo test --test <name>`" not just "Test: `cargo test`". The targeted command saves ~90% of test output tokens.

- **R12: PostToolUse hook filters test output** — check if hooks exist that grep for FAIL/ERROR only, suppressing passing test output. A 5000-line test run filtered to 10 failure lines = 99% token savings.

---

## Stage 6: Cross-session efficiency

### What burns tokens across sessions:
- Starting every session from scratch — no memory of prior work
- Repeating the same exploration patterns
- No cached understanding of architecture

### New rules to add:
- **R13: .claude/memory/ exists with project memories** — Claude Code's memory system persists across sessions. Projects that actively use it avoid re-exploration.

- **R14: Architecture decision records exist** — `docs/adr/` or equivalent. Claude reads one ADR to understand why code is structured a certain way, instead of reverse-engineering intent from code.

---

## Priority for implementation

### Highest token-saving impact:
1. **R11: Targeted test commands** — saves 2000+ tokens per test run
2. **R3: Directory index / MANIFEST.md** — saves 500+ tokens per exploration
3. **R12: Test output filtering hooks** — saves 3000+ tokens per test run
4. **R5: Exports at top of files** — saves 100+ tokens per file read
5. **R1: No CLAUDE.md/README duplication** — saves tokens on every request

### Medium impact:
6. R7: Co-located related code
7. R8: Config at top of file
8. R9: Clear function boundaries
9. R2: Narrow rule path scopes
10. R14: Architecture decision records

### Lower impact (but good practice):
11. R4: Entry point per module
12. R6: No circular dependencies
13. R10: No mixed generated+handwritten
14. R13: Memory directory exists
