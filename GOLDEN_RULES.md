# Claude Native: Golden Rules & Best Practices

A project is **Claude Native** when it is structurally optimized for AI-assisted development —
minimizing token cost, maximizing Claude's ability to navigate/understand/modify the codebase,
and leveraging the full Claude Code ecosystem.

---

## Scoring Dimensions

| Dimension | Weight | What it measures |
|-----------|--------|-----------------|
| **Foundation** | 25% | CLAUDE.md, .claudeignore, .claude/ directory setup |
| **Context Efficiency** | 25% | File sizes, modularity, token waste reduction |
| **Navigation & Discoverability** | 20% | Structure, naming, entry points |
| **Tooling & Automation** | 15% | MCP, hooks, skills, settings, permissions |
| **Code Quality for AI** | 15% | Types, tests, consistency, documentation |

---

## 1. FOUNDATION (25%)

### Rule 1.1 — CLAUDE.md must exist
- **Check**: `CLAUDE.md` or `.claude/CLAUDE.md` exists at project root.
- **Why**: This is Claude's persistent memory for your project. Without it, every session starts from zero.
- **Severity**: CRITICAL

### Rule 1.2 — CLAUDE.md is concise (<200 lines)
- **Check**: Line count of root CLAUDE.md < 200.
- **Why**: CLAUDE.md loads into context on every single request. Every extra line costs tokens on every message. A 500-line CLAUDE.md wastes ~300 tokens per request.
- **Severity**: HIGH
- **Fix**: Move specialized instructions to `.claude/rules/` or skills.

### Rule 1.3 — CLAUDE.md contains actionable instructions, not documentation
- **Check**: CLAUDE.md should contain build/test commands, code style rules, architectural decisions, gotchas. Should NOT contain tutorials, API docs, changelogs, or information derivable from code.
- **Why**: Claude can read code itself. CLAUDE.md should tell Claude what it *can't* infer.
- **Severity**: MEDIUM
- **Anti-patterns**:
  - File-by-file descriptions of the codebase
  - Copy-pasted README content
  - Standard language conventions Claude already knows
  - Frequently changing information (team assignments, sprint goals)

### Rule 1.4 — CLAUDE.md has build/test commands
- **Check**: CLAUDE.md contains runnable commands for building and testing.
- **Why**: Claude needs to know how to verify its own changes. Without this, it guesses or asks.
- **Severity**: HIGH

### Rule 1.5 — .claudeignore exists and excludes noise
- **Check**: `.claudeignore` file exists at project root.
- **Why**: Without it, Glob and Grep hit thousands of irrelevant files (node_modules, build artifacts, vendor dirs), wasting tokens on every search.
- **Severity**: HIGH
- **Must exclude**: dependencies (node_modules, .venv, vendor), build outputs (dist, build, target, .next), generated files (coverage, *.log), secrets (.env, *.pem, *.key), OS files (.DS_Store)

### Rule 1.6 — .claude/ directory exists with project settings
- **Check**: `.claude/` directory exists.
- **Why**: This is the home for project-level Claude configuration — settings, hooks, skills, rules.
- **Severity**: MEDIUM

### Rule 1.7 — .claude/settings.json exists with permissions
- **Check**: `.claude/settings.json` exists and contains `permissions.allow` entries.
- **Why**: Pre-approved permissions (e.g., `Bash(npm test:*)`) reduce interruptions and let Claude work autonomously on safe operations.
- **Severity**: MEDIUM

---

## 2. CONTEXT EFFICIENCY (25%)

### Rule 2.1 — No mega-files (>500 lines)
- **Check**: No source files exceed 500 lines.
- **Why**: Claude reads entire files. A 2000-line file costs ~2000 tokens even if Claude only needs 10 lines. Smaller files = cheaper reads, better targeted edits.
- **Severity**: HIGH
- **Threshold**: Warning at >300 lines, error at >500 lines.

### Rule 2.2 — No mega-functions (>80 lines)
- **Check**: No single function/method exceeds 80 lines of complex logic.
- **Why**: Long functions force Claude to read more context to understand behavior. They're also harder for Claude to edit correctly (higher chance of introducing bugs).
- **Severity**: MEDIUM
- **Threshold**: Warning at >50 lines, error at >80 lines.
- **Exceptions**: Registry/list functions (e.g., `vec![Box::new(...)]`), match tables, and config builders are excluded — they're long but not complex. Brace-counting must skip string/char literals (`"{"`, `'{'`) to avoid false positives.

### Rule 2.3 — Lock files are ignored
- **Check**: Lock files (package-lock.json, yarn.lock, Cargo.lock, Gemfile.lock, poetry.lock, go.sum) are in .claudeignore.
- **Why**: Lock files can be 10,000+ lines. Claude should never read them — they provide zero useful context.
- **Severity**: HIGH

### Rule 2.4 — Generated/compiled files are ignored
- **Check**: Build outputs, compiled files, and generated code are in .claudeignore.
- **Why**: These files are derived from source. Reading them wastes tokens and can confuse Claude.
- **Severity**: HIGH

### Rule 2.5 — No secrets in the repository
- **Check**: No .env files, API keys, tokens, or credentials committed to git.
- **Why**: Security risk + Claude might read and echo them back in conversation. Also wastes tokens.
- **Severity**: CRITICAL

### Rule 2.6 — README exists and is concise
- **Check**: README.md exists and is <300 lines.
- **Why**: Claude reads README to understand the project. A concise README = fast onboarding. A 1000-line README = wasted tokens.
- **Severity**: LOW

### Rule 2.7 — Subdirectory CLAUDE.md files for large projects
- **Check**: In monorepos or projects with >20 source files, subdirectory CLAUDE.md files exist for major modules.
- **Why**: Subdirectory CLAUDE.md files load on-demand (only when Claude works in that directory), unlike the root CLAUDE.md which loads always.
- **Severity**: LOW (MEDIUM for monorepos)

---

## 3. NAVIGATION & DISCOVERABILITY (20%)

### Rule 3.1 — Clear directory structure
- **Check**: Source code is organized into logical directories (not flat dump of files at root).
- **Why**: Claude uses Glob patterns to find files. A well-organized structure means more precise searches.
- **Severity**: MEDIUM
- **Anti-patterns**: >15 source files in a single directory, >4 levels of nesting, mixed concerns in one folder.

### Rule 3.2 — Consistent file naming conventions
- **Check**: Files follow a consistent naming pattern (snake_case, kebab-case, or PascalCase — but consistent).
- **Why**: Consistency lets Claude predict where things are. If handlers are sometimes `userHandler.ts` and sometimes `handle-user.ts`, Claude wastes tokens searching.
- **Severity**: MEDIUM

### Rule 3.3 — Entry points are obvious
- **Check**: Main entry point is clearly identifiable (main.rs, index.ts, app.py, main.go, etc.) or documented in CLAUDE.md.
- **Why**: Claude starts exploration from entry points. An obvious entry point saves a round of searching.
- **Severity**: LOW

### Rule 3.4 — Module boundaries are clear
- **Check**: Each major directory has a clear public API (index file, mod.rs, __init__.py, etc.).
- **Why**: Clear boundaries let Claude understand what a module exports without reading every file in it.
- **Severity**: MEDIUM

### Rule 3.5 — Tests are co-located or in a predictable location
- **Check**: Tests follow a consistent pattern — either co-located (`*.test.ts` next to source) or in a parallel `tests/` directory.
- **Why**: Claude needs to find and run tests to verify changes. Unpredictable test locations = wasted search tokens.
- **Severity**: MEDIUM

### Rule 3.6 — No deeply nested directories (>4 levels)
- **Check**: Maximum directory nesting depth from project root is ≤4.
- **Why**: Deep nesting makes Glob patterns expensive and navigation confusing for both Claude and humans.
- **Severity**: LOW

### Rule 3.7 — Descriptive directory and file names
- **Check**: No single-character or cryptic directory names (e.g., `src/u/` instead of `src/utils/`).
- **Why**: Claude uses names to decide what to read. Descriptive names reduce false reads.
- **Severity**: LOW

---

## 4. TOOLING & AUTOMATION (15%)

### Rule 4.1 — MCP servers configured for external services
- **Check**: If project uses external services (GitHub, databases, APIs), `.claude/.mcp.json` exists with relevant servers.
- **Why**: MCP servers give Claude direct access to external tools without shell workarounds. More reliable, often cheaper.
- **Severity**: LOW

### Rule 4.2 — Hooks for auto-formatting
- **Check**: PostToolUse hook exists for Edit/Write that runs formatter (prettier, black, rustfmt, etc.).
- **Why**: Without auto-format hooks, Claude's edits may not match project style, triggering linter errors and follow-up fixes (wasting tokens).
- **Severity**: MEDIUM

### Rule 4.3 — Hooks for dangerous operation protection
- **Check**: PreToolUse hooks exist to block editing of sensitive files (.env, lock files, CI configs).
- **Why**: Prevention is cheaper than correction. Hooks deterministically block bad actions.
- **Severity**: LOW

### Rule 4.4 — Custom skills for repetitive workflows
- **Check**: `.claude/skills/` directory exists with project-specific skills.
- **Why**: Skills load on-demand (not at session start like CLAUDE.md). They encode complex multi-step workflows that Claude can invoke by name.
- **Severity**: LOW

### Rule 4.5 — Permission allow-list is configured
- **Check**: `.claude/settings.json` has `permissions.allow` for common safe commands (test runners, build tools, git operations).
- **Why**: Every permission prompt interrupts Claude's flow and requires user interaction. Pre-approving safe commands makes Claude autonomous.
- **Severity**: MEDIUM

### Rule 4.6 — .claude/rules/ for path-scoped instructions
- **Check**: `.claude/rules/` directory exists with topic-specific rule files that use `paths:` frontmatter.
- **Why**: Rules load only when Claude works with matching files. This keeps the always-loaded CLAUDE.md small while providing deep guidance where needed.
- **Severity**: LOW (HIGH for large projects)

---

## 5. CODE QUALITY FOR AI (15%)

### Rule 5.1 — Type annotations exist
- **Check**: For dynamically typed languages (Python, JS), type annotations or TypeScript/mypy are used.
- **Why**: Types give Claude a contract to work with. Without types, Claude must read entire function bodies to understand data shapes. Types reduce hallucinated return values.
- **Severity**: MEDIUM

### Rule 5.2 — Tests exist
- **Check**: Test files exist and a test command is runnable.
- **Why**: Tests are Claude's primary verification mechanism. Without tests, Claude can't validate its own changes, leading to silent bugs.
- **Severity**: HIGH

### Rule 5.3 — Tests have descriptive names
- **Check**: Test names describe behavior, not just "test1", "test2".
- **Why**: Claude reads test names to understand expected behavior. Good test names = less code reading needed.
- **Severity**: LOW

### Rule 5.4 — Consistent patterns across codebase
- **Check**: Error handling, data access, and API patterns are consistent (not 3 different ways to query the database).
- **Why**: Consistency lets Claude pattern-match. If it sees one handler, it can write the next one correctly. Inconsistency means Claude picks the wrong pattern 33% of the time.
- **Severity**: MEDIUM

### Rule 5.5 — Comments explain "why", not "what"
- **Check**: Comments are present for non-obvious business logic, edge cases, and workarounds.
- **Why**: Claude understands what code does by reading it. Comments should explain *why* it does it — business context, external constraints, known gotchas.
- **Severity**: LOW

### Rule 5.6 — No dead code
- **Check**: No large blocks of commented-out code, unused imports, or dead functions.
- **Why**: Dead code confuses Claude. It might try to use deprecated functions, or waste tokens reading code that does nothing.
- **Severity**: LOW

### Rule 5.7 — Dependencies are documented
- **Check**: Package manifest exists (package.json, Cargo.toml, requirements.txt, go.mod) and is up to date.
- **Why**: Claude uses manifests to understand what libraries are available. Missing or outdated manifests lead to import errors.
- **Severity**: MEDIUM

### Rule 5.8 — CI/CD pipeline exists
- **Check**: CI configuration file exists (.github/workflows/, .gitlab-ci.yml, Jenkinsfile, etc.).
- **Why**: CI provides automated verification. Claude can reference CI configuration to understand build/test/deploy processes.
- **Severity**: LOW

---

## Quick Reference: The 10 Commandments of Claude Native

1. **Thou shalt have a CLAUDE.md** — concise, actionable, <200 lines.
2. **Thou shalt ignore the noise** — .claudeignore for deps, builds, generated files.
3. **Thou shalt keep files small** — <500 lines per file, <80 lines per function.
4. **Thou shalt name things clearly** — descriptive, consistent naming everywhere.
5. **Thou shalt write tests** — Claude's primary way to verify its own work.
6. **Thou shalt be consistent** — one pattern per concern, applied everywhere.
7. **Thou shalt use types** — contracts Claude can reason about.
8. **Thou shalt structure logically** — clear directories, obvious entry points.
9. **Thou shalt automate guardrails** — hooks for formatting, permissions for safe commands.
10. **Thou shalt separate concerns** — rules/ for deep guidance, skills for workflows, CLAUDE.md for essentials.

---

## Scoring Formula

```
Total Score = (Foundation × 0.25) + (Context × 0.25) + (Navigation × 0.20) + (Tooling × 0.15) + (CodeQuality × 0.15)
```

Each dimension scores 0-100 based on rules passed:
- CRITICAL rule failed = dimension score capped at 30
- HIGH rule failed = -20 points from dimension
- MEDIUM rule failed = -10 points from dimension
- LOW rule failed = -5 points from dimension

### Grade Scale

| Score | Grade | Meaning |
|-------|-------|---------|
| 90-100 | A+ | Fully Claude Native — optimized for AI-assisted development |
| 80-89 | A | Claude Native — well set up with minor improvements possible |
| 70-79 | B | Claude Friendly — good foundation, notable gaps |
| 60-69 | C | Claude Compatible — works but significant optimization possible |
| 40-59 | D | Claude Hostile — major friction, high token waste |
| 0-39 | F | Not Claude Native — needs fundamental restructuring |

---

## 6. PROJECT TYPE EDGE CASES

The scanner must detect the project type and adjust rules, thresholds, and suggestions accordingly.

### Detection Strategy
1. **Monorepo signals**: Multiple `package.json`/`Cargo.toml`/`go.mod` files, workspace configs (`pnpm-workspace.yaml`, `[workspace]` in Cargo.toml, Nx/Turborepo/Lerna configs), top-level `packages/` or `apps/` directories.
2. **Micro-repo signals**: Single package manifest, single concern, <50 source files, no workspace config, often a library/package/microservice.

---

### 6.1 — Monorepo Edge Cases

Monorepos have fundamentally different needs. Many rules change in severity or behavior.

#### Rule M1 — Root CLAUDE.md must be a thin orchestrator
- **Check**: Root CLAUDE.md exists but is <100 lines (stricter than normal). Contains only cross-cutting instructions (git workflow, CI, shared conventions). Does NOT contain package-specific instructions.
- **Why**: In a monorepo, the root CLAUDE.md loads on EVERY request across ALL packages. Package-specific instructions bloat every session even when working in one package.
- **Severity**: HIGH
- **Fix**: Move package-specific instructions to `packages/<name>/CLAUDE.md`.

#### Rule M2 — Every package/app has its own CLAUDE.md
- **Check**: Each directory under `packages/`, `apps/`, `services/`, `libs/` (or equivalent workspace members) has its own CLAUDE.md.
- **Why**: Subdirectory CLAUDE.md files load on-demand. This gives Claude deep context for the package being worked on without paying the token cost for all other packages.
- **Severity**: HIGH

#### Rule M3 — .claudeignore accounts for all workspace outputs
- **Check**: .claudeignore covers build outputs for ALL packages (e.g., `packages/*/dist/`, `apps/*/build/`, `**/node_modules/`).
- **Why**: Monorepos multiply the noise problem. 10 packages × unignored dist/ = 10x the wasted tokens.
- **Severity**: HIGH

#### Rule M4 — .claude/rules/ uses path-scoped rules per package
- **Check**: `.claude/rules/` contains rules with `paths:` frontmatter scoping them to specific packages.
- **Why**: A rule about React components shouldn't load when Claude is editing a backend Go service in the same monorepo.
- **Severity**: MEDIUM
- **Example**:
  ```markdown
  # .claude/rules/frontend-react.md
  ---
  paths:
    - "apps/web/**"
    - "packages/ui/**"
  ---
  - Use React Server Components by default
  - CSS modules, not Tailwind
  ```

#### Rule M5 — Per-package test commands documented
- **Check**: Each package's CLAUDE.md or root CLAUDE.md documents how to test individual packages (not just `npm test` at root which runs everything).
- **Why**: Running ALL tests in a monorepo after a single-package change wastes minutes and thousands of tokens on irrelevant output. Claude needs to know `cd packages/auth && npm test`.
- **Severity**: HIGH

#### Rule M6 — Workspace dependency graph is navigable
- **Check**: Internal dependencies between packages are declared in manifests (not implicit). Workspace references use proper syntax (`workspace:*`, `path = "../"`, etc.).
- **Why**: Claude needs to trace cross-package imports. Implicit dependencies mean Claude reads the wrong version or misses breaking changes.
- **Severity**: MEDIUM

#### Rule M7 — Shared tooling config at root
- **Check**: Shared configs (tsconfig.base.json, .eslintrc, rustfmt.toml, etc.) exist at root and are extended by packages.
- **Why**: Claude uses these to understand code style. Per-package configs that contradict each other confuse Claude.
- **Severity**: LOW

#### Rule M8 — Mega-file threshold is stricter
- **Check**: Source files >300 lines trigger warnings (vs 500 for standard repos).
- **Why**: In monorepos, Claude works across packages. Smaller files per package = less context consumed when cross-referencing. The total codebase is already large; individual files must compensate.
- **Severity**: MEDIUM
- **Adjusted thresholds**: Warning at >200 lines, error at >300 lines.

#### Rule M9 — CI runs package-scoped checks
- **Check**: CI configuration supports running checks per package (not only full-repo builds). Look for path filters, affected-package detection (Nx affected, Turborepo filter, etc.).
- **Why**: Claude references CI to understand verification. If CI only runs everything, Claude can't selectively verify changes.
- **Severity**: LOW

#### Monorepo Scoring Adjustments
- Rules M1-M5 are **unique to monorepos** and replace some base rules:
  - Rule 1.2 (CLAUDE.md <200 lines) → M1 (root <100 lines) + M2 (per-package CLAUDE.md)
  - Rule 2.7 severity upgrades from LOW → HIGH
  - Rule 4.6 severity upgrades from LOW → MEDIUM
- Base dimension weights shift:
  - Foundation: 25% → **30%** (more configuration surface area)
  - Context Efficiency: 25% → **25%** (unchanged, still critical)
  - Navigation: 20% → **20%** (unchanged)
  - Tooling: 15% → **15%** (unchanged)
  - Code Quality: 15% → **10%** (per-package, less weight at repo level)

---

### 6.2 — Micro-repo Edge Cases

Micro-repos (single-purpose repos: one library, one microservice, one CLI tool) have different optimization points.

#### Rule μ1 — CLAUDE.md can be minimal but must exist
- **Check**: CLAUDE.md exists with at least: project purpose (1 line), build command, test command.
- **Why**: Micro-repos are simple enough that Claude can often infer structure. But build/test commands are still essential.
- **Severity**: HIGH
- **Minimum viable CLAUDE.md** (3 lines):
  ```markdown
  # Auth Service — JWT-based authentication microservice
  Build: `cargo build`
  Test: `cargo test`
  ```

#### Rule μ2 — .claudeignore is still required
- **Check**: .claudeignore exists even for small projects.
- **Why**: Even a small Node.js project has node_modules/ with 50,000+ files. The noise ratio is actually WORSE in micro-repos (small source, large deps).
- **Severity**: HIGH

#### Rule μ3 — Subdirectory CLAUDE.md files are unnecessary
- **Check**: No subdirectory CLAUDE.md files exist (not penalized if absent).
- **Why**: In a micro-repo with <50 files, the root CLAUDE.md provides sufficient context. Subdirectory files add maintenance overhead for no benefit.
- **Severity**: N/A (skip this check)

#### Rule μ4 — .claude/rules/ is optional
- **Check**: Not penalized if `.claude/rules/` doesn't exist.
- **Why**: Small codebases don't need path-scoped rules. The root CLAUDE.md covers everything.
- **Severity**: N/A (skip this check)

#### Rule μ5 — Skills are optional
- **Check**: Not penalized if `.claude/skills/` doesn't exist.
- **Why**: Micro-repos rarely have complex multi-step workflows worth encoding as skills.
- **Severity**: N/A (skip this check)

#### Rule μ6 — Higher tolerance for flat structure
- **Check**: Flat directory structure (all source files in `src/` with no subdirectories) is acceptable if <20 source files.
- **Why**: A 10-file library doesn't need `src/utils/`, `src/handlers/`, `src/models/`. Forced structure adds friction without helping Claude.
- **Severity**: N/A (relax Rule 3.1)

#### Rule μ7 — README is the primary documentation
- **Check**: README.md exists with: purpose, installation, usage, API surface.
- **Why**: In micro-repos, the README IS the documentation. Claude reads it to understand the library/service contract. This is more important than in monorepos where internal docs may exist.
- **Severity**: MEDIUM (upgraded from LOW)

#### Rule μ8 — Comprehensive tests are more critical
- **Check**: Test coverage is proportionally higher. For a library with 10 source files, at least 5 test files should exist.
- **Why**: Micro-repos are often consumed by other projects. Claude's changes must not break consumers. Tests are the only safety net — no internal team catches regressions.
- **Severity**: HIGH (upgraded from base)

#### Rule μ9 — Package manifest is complete
- **Check**: Manifest has: name, version, description, license, repository, keywords, entry points.
- **Why**: Micro-repos are often published packages. Claude uses the manifest to understand the package contract, exports, and how consumers use it.
- **Severity**: MEDIUM

#### Rule μ10 — Examples directory or inline examples
- **Check**: `examples/` directory exists OR README contains usage examples.
- **Why**: Claude uses examples to understand intended usage patterns. For a library, examples are more useful than architecture docs.
- **Severity**: LOW

#### Micro-repo Scoring Adjustments
- Several tooling rules are relaxed (μ3, μ4, μ5 — skip checks)
- Several quality rules are upgraded (μ8 — tests, μ7 — README)
- Base dimension weights shift:
  - Foundation: 25% → **20%** (less configuration needed)
  - Context Efficiency: 25% → **20%** (small codebase, less waste risk)
  - Navigation: 20% → **15%** (flat structure is fine)
  - Tooling: 15% → **15%** (unchanged)
  - Code Quality: 15% → **30%** (quality matters most for consumed packages)

---

### 6.3 — Mobile App Edge Cases (Flutter, React Native, iOS, Android)

Mobile projects have massive generated/platform directories that dwarf the actual source code.

#### Rule MOB1 — Platform build directories are ignored
- **Check**: .claudeignore includes platform build artifacts.
- **Why**: Mobile builds generate hundreds of MB of artifacts. A single `ios/Pods/` directory can contain 50,000+ files.
- **Severity**: CRITICAL
- **Must ignore by platform**:
  - **Flutter**: `build/`, `.dart_tool/`, `ios/Pods/`, `android/.gradle/`, `android/build/`, `.pub-cache/`
  - **React Native**: `node_modules/`, `ios/Pods/`, `android/.gradle/`, `android/build/`, `.expo/`
  - **iOS (Swift)**: `Pods/`, `DerivedData/`, `.build/`, `*.xcodeproj/xcuserdata/`
  - **Android (Kotlin)**: `.gradle/`, `build/`, `**/intermediates/`, `**/generated/`

#### Rule MOB2 — Generated code is ignored
- **Check**: Code generation outputs are in .claudeignore.
- **Why**: Mobile projects use heavy codegen (build_runner, protobuf, R.java). Generated files can be 10-50KB each and provide zero insight Claude can't get from the source spec.
- **Severity**: HIGH
- **Must ignore**:
  - Flutter: `*.g.dart`, `*.freezed.dart`, `*.config.dart`, `*.pb.dart`
  - React Native: `*.pb.js`, minified `.bundle` files
  - iOS: `*.generated.swift`, `*.pb.swift`
  - Android: `BuildConfig.kt`, `R.java`/`R.kt`, `**/databinding/**`

#### Rule MOB3 — CLAUDE.md documents platform-specific commands
- **Check**: CLAUDE.md contains build/test commands per platform, not just generic ones.
- **Why**: Mobile projects have multiple build targets. `flutter test` is different from `flutter test integration_test/`. Claude needs to know which command to use when.
- **Severity**: HIGH
- **Example**:
  ```markdown
  # Quick checks
  Analyze: `flutter analyze`
  Unit test: `flutter test`
  Integration test: `flutter test integration_test/`
  # Don't use `flutter build` during development
  ```

#### Rule MOB4 — Binary assets are excluded from search
- **Check**: Image, audio, font, and model assets are in .claudeignore.
- **Why**: `assets/images/`, `Resources/`, `res/drawable/` contain binary files Claude can't read but will still hit during Glob/Grep searches.
- **Severity**: MEDIUM
- **Exclude**: `*.png`, `*.jpg`, `*.svg`, `*.ttf`, `*.otf`, `*.mp3`, `*.wav`, `*.fbx`

#### Rule MOB5 — Use lightweight verification commands
- **Check**: CLAUDE.md prioritizes fast verification over full builds.
- **Why**: Full mobile builds take 2-5 minutes and produce massive output. `flutter analyze` takes seconds. Claude should use the fast path.
- **Severity**: MEDIUM

#### Mobile Detection Heuristic
```
IF pubspec.yaml exists → FLUTTER
IF react-native in package.json dependencies → REACT_NATIVE
IF *.xcodeproj exists AND no Flutter/RN markers → IOS_NATIVE
IF build.gradle.kts with android plugin AND no Flutter/RN markers → ANDROID_NATIVE
```

---

### 6.4 — Frontend SPA Edge Cases (Next.js, React, Vue, Angular)

Frontend projects have heavy tooling that generates enormous build artifacts.

#### Rule FE1 — Build cache directories are ignored
- **Check**: .claudeignore includes framework build caches.
- **Why**: `.next/` alone can be 500MB+. These regenerate on every build and contain zero useful source context.
- **Severity**: CRITICAL
- **Must ignore**: `.next/`, `.nuxt/`, `.angular/`, `dist/`, `build/`, `.vercel/`, `.turbo/`, `.cache/`

#### Rule FE2 — Source maps are ignored
- **Check**: `*.map` files are in .claudeignore.
- **Why**: Source maps can be larger than the original source. Claude never needs them.
- **Severity**: HIGH

#### Rule FE3 — .env files have an .env.example companion
- **Check**: If `.env` exists (or `.env.local`, `.env.production`), a `.env.example` also exists documenting required variables.
- **Why**: Claude should read `.env.example` (safe, committed) to understand the env shape, never `.env` (secrets). Without `.env.example`, Claude has no way to know what env vars exist.
- **Severity**: HIGH

#### Rule FE4 — CLAUDE.md prioritizes type-check over build
- **Check**: CLAUDE.md lists `tsc --noEmit` or equivalent type-check before `npm run build`.
- **Why**: Type-checking catches 90% of errors in 5 seconds. A full build takes 2+ minutes. Claude should always type-check first.
- **Severity**: MEDIUM

#### Rule FE5 — Component tests are co-located
- **Check**: Component test files are next to their components (`Button.test.tsx` beside `Button.tsx`).
- **Why**: Frontend projects have many small component files. Co-located tests let Claude find the test instantly without searching.
- **Severity**: LOW

#### Frontend Detection Heuristic
```
IF next.config.* exists → NEXTJS
IF nuxt.config.* exists → NUXT
IF angular.json exists → ANGULAR
IF vite.config.* OR vue.config.* exists → VUE/VITE
IF package.json has react-scripts → CREATE_REACT_APP
```

---

### 6.5 — Backend API Edge Cases (Django, Rails, Express, Go, Rust)

Backend projects have database state, migration histories, and verbose logging.

#### Rule BE1 — Migration history is manageable
- **Check**: Total number of migration files is <200. If >200, older migrations are squashed.
- **Why**: Claude may read migration files to understand schema. 500 Django migrations = thousands of tokens wasted. Squashed migrations keep token cost low.
- **Severity**: MEDIUM
- **Threshold**: Warning at >100 migrations, error at >200.

#### Rule BE2 — Database state files are ignored
- **Check**: Database dumps, SQLite files, and seed data >1MB are in .claudeignore.
- **Why**: `db.sqlite3` can be 100MB+. Seed data files with test records are massive. Claude reads schema from migrations/models, not data files.
- **Severity**: HIGH
- **Exclude**: `*.sqlite3`, `*.sql` (dumps), `db/seeds/*.json` (large seed data)

#### Rule BE3 — Virtual environments are ignored
- **Check**: `.venv/`, `venv/`, `vendor/`, `__pycache__/` are in .claudeignore.
- **Why**: Python's `venv` contains the entire standard library copy. Go's `vendor/` can be 100MB+.
- **Severity**: CRITICAL

#### Rule BE4 — CLAUDE.md documents the ORM and data access pattern
- **Check**: CLAUDE.md explains which ORM/query pattern is used and how.
- **Why**: Backend projects often have 3+ ways to query data (raw SQL, ORM, query builder). Claude picks the wrong one 33% of the time without guidance.
- **Severity**: MEDIUM

#### Rule BE5 — API spec exists (OpenAPI/Swagger)
- **Check**: `openapi.yaml`, `swagger.json`, or equivalent API spec file exists.
- **Why**: Claude uses API specs to understand endpoints, request/response shapes, and validation rules without reading every handler.
- **Severity**: LOW

#### Rule BE6 — Log files are ignored
- **Check**: `logs/`, `*.log`, `tmp/` are in .claudeignore.
- **Why**: A single log file can be 100MB. Claude should never read raw logs — use filtered commands instead.
- **Severity**: HIGH

#### Backend Detection Heuristic
```
IF manage.py OR django in requirements → DJANGO
IF Gemfile with rails → RAILS
IF express in package.json dependencies → EXPRESS
IF go.mod exists AND cmd/ directory exists → GO_SERVICE
IF Cargo.toml with actix/axum/rocket → RUST_SERVICE
IF mix.exs with phoenix → PHOENIX
```

---

### 6.6 — Infrastructure-as-Code Edge Cases (Terraform, Kubernetes, Pulumi)

IaC projects have state files containing secrets and verbose plan outputs.

#### Rule IAC1 — State files are NEVER readable
- **Check**: `*.tfstate`, `*.tfstate.backup`, `.terraform/` are in .claudeignore. Hook blocks reading these files.
- **Why**: Terraform state contains ALL resource attributes including passwords, tokens, and private keys. Reading these is a security breach AND a massive token waste (10-50MB).
- **Severity**: CRITICAL
- **Recommended hook**:
  ```json
  {
    "hooks": {
      "PreToolUse": [{
        "matcher": "Read",
        "hooks": [{
          "type": "command",
          "command": "echo $TOOL_INPUT | jq -r '.file_path' | grep -qE '\\.(tfstate|tfstate\\.backup)$' && echo 'BLOCKED: State files contain secrets' >&2 && exit 2 || exit 0"
        }]
      }]
    }
  }
  ```

#### Rule IAC2 — Plan output is filtered
- **Check**: CLAUDE.md instructs to filter `terraform plan` or `helm template` output.
- **Why**: Raw plan output for large environments is 5,000-50,000 lines. Claude should see resource changes only, not full diffs.
- **Severity**: HIGH

#### Rule IAC3 — Provider/plugin cache is ignored
- **Check**: `.terraform/`, `node_modules/` (CDK), `.pulumi/` are in .claudeignore.
- **Why**: Provider plugins are large binaries that Claude can't read.
- **Severity**: HIGH

#### Rule IAC4 — Secrets are managed externally
- **Check**: No `terraform.tfvars` with real values committed. Variables reference vault/SSM/env vars.
- **Why**: IaC is the highest risk for secret exposure since infrastructure definitions inherently reference credentials.
- **Severity**: CRITICAL

#### Rule IAC5 — Module structure uses variables.tf/outputs.tf convention
- **Check**: Each Terraform module directory has `variables.tf`, `outputs.tf`, `main.tf`.
- **Why**: This is a universal Terraform convention. Claude navigates modules by reading `variables.tf` first (contract), then `main.tf` (implementation). Without this convention, Claude must read everything.
- **Severity**: MEDIUM

#### Rule IAC6 — Kubernetes manifests use Helm or Kustomize (not raw YAML)
- **Check**: If >5 Kubernetes YAML files exist, they use Helm templates or Kustomize overlays.
- **Why**: Raw YAML manifests have massive boilerplate repetition. Templated manifests are DRYer — fewer tokens for Claude to read.
- **Severity**: LOW

#### IaC Detection Heuristic
```
IF *.tf files exist → TERRAFORM
IF Chart.yaml exists → HELM
IF kustomization.yaml exists → KUSTOMIZE
IF Pulumi.yaml exists → PULUMI
IF cdk.json exists → AWS_CDK
IF serverless.yml OR template.yaml (SAM) exists → SERVERLESS
```

---

### 6.7 — Serverless / FaaS Edge Cases (Lambda, CloudFlare Workers, Vercel Functions)

Serverless projects have many small functions with shared layers and massive deployment artifacts.

#### Rule SLS1 — Deployment artifacts are ignored
- **Check**: `.aws-sam/`, `.serverless/`, `.vercel/`, `cdk.out/` are in .claudeignore.
- **Why**: `sam build` generates a full CloudFormation template (10,000+ lines). Deployment packages contain zipped node_modules.
- **Severity**: HIGH

#### Rule SLS2 — Each function has focused scope
- **Check**: Individual function handlers are <100 lines.
- **Why**: Serverless functions should be small by design. If a handler is 500 lines, it's doing too much — Claude will struggle to understand and modify it.
- **Severity**: MEDIUM
- **Threshold**: Warning at >50 lines, error at >100 lines (stricter than standard).

#### Rule SLS3 — Shared layers/dependencies are documented
- **Check**: CLAUDE.md explains what's in Lambda layers or shared dependencies.
- **Why**: Layer contents are often pre-built binaries Claude can't inspect. Without documentation, Claude doesn't know what's available at runtime.
- **Severity**: MEDIUM

#### Rule SLS4 — Test events exist
- **Check**: `events/` or `test-events/` directory exists with sample event JSON files.
- **Why**: Serverless functions are triggered by events (API Gateway, SQS, S3). Claude needs sample events to test locally with `sam local invoke`.
- **Severity**: MEDIUM

#### Rule SLS5 — Cloud credentials are never in the repo
- **Check**: No `.aws/`, `credentials`, or cloud auth tokens in the project.
- **Why**: Serverless projects have direct cloud access. Leaked credentials = full cloud compromise.
- **Severity**: CRITICAL

#### Serverless Detection Heuristic
```
IF serverless.yml exists → SERVERLESS_FRAMEWORK
IF template.yaml with Transform: AWS::Serverless → AWS_SAM
IF wrangler.toml exists → CLOUDFLARE_WORKERS
IF vercel.json with functions config → VERCEL_FUNCTIONS
IF netlify.toml with functions directory → NETLIFY_FUNCTIONS
```

---

### 6.8 — Data Science / ML Edge Cases (Jupyter, PyTorch, TensorFlow)

ML projects mix code with large binary artifacts, notebooks, and datasets.

#### Rule DS1 — Notebooks are NOT primary source
- **Check**: Core logic lives in `.py` files, not `.ipynb` notebooks.
- **Why**: Jupyter notebooks contain cell outputs (images, tables, HTML) that inflate token cost 10-50x. A 100-line notebook with outputs can cost 5,000+ tokens. `.py` files cost ~100 tokens for the same logic.
- **Severity**: HIGH

#### Rule DS2 — Model files are ignored
- **Check**: `*.pkl`, `*.h5`, `*.pth`, `*.onnx`, `*.safetensors`, `*.bin` (model weights) are in .claudeignore.
- **Why**: Model files are binary (100MB-10GB). Claude can't read them and will error/waste tokens trying.
- **Severity**: CRITICAL

#### Rule DS3 — Dataset files are ignored
- **Check**: `data/`, `*.csv` (>1MB), `*.parquet`, `*.feather`, `*.arrow` are in .claudeignore.
- **Why**: Datasets can be GB-scale. Claude should sample data via commands (`head -20 data.csv`), not read entire files.
- **Severity**: CRITICAL

#### Rule DS4 — CLAUDE.md documents the experiment workflow
- **Check**: CLAUDE.md explains: how to run training, how to evaluate, what metrics matter, where results are stored.
- **Why**: ML projects have non-obvious workflows (preprocess → train → evaluate → compare). Without guidance, Claude runs the wrong step or uses stale data.
- **Severity**: MEDIUM

#### Rule DS5 — Requirements pin exact versions
- **Check**: `requirements.txt` or `pyproject.toml` pins exact versions (`torch==2.1.0`, not `torch>=2.0`).
- **Why**: ML dependencies have complex compatibility (CUDA versions, framework versions). Unpinned versions cause "it works on my machine" failures.
- **Severity**: MEDIUM

#### Rule DS6 — Config files separate hyperparameters from code
- **Check**: Hyperparameters live in config files (YAML, JSON, TOML), not hardcoded in scripts.
- **Why**: Claude can modify a config file without understanding the training loop. Hardcoded params require Claude to read and understand the full script.
- **Severity**: LOW

#### Data Science Detection Heuristic
```
IF *.ipynb files exist AND (torch OR tensorflow OR sklearn in requirements) → ML_PROJECT
IF *.ipynb files exist AND no ML frameworks → DATA_ANALYSIS
IF dvc.yaml OR .dvc/ exists → DVC_ML_PROJECT
IF MLproject file exists → MLFLOW_PROJECT
```

---

### 6.9 — Code Generation Heavy Edge Cases (Protobuf, GraphQL, OpenAPI, Prisma)

Projects where a significant portion of code is machine-generated from specs.

#### Rule GEN1 — Generated code directories are ignored
- **Check**: All generated output directories are in .claudeignore.
- **Why**: Generated code is derived from specs. Reading it wastes tokens AND confuses Claude (it might edit generated code that gets overwritten).
- **Severity**: CRITICAL
- **Common patterns**: `src/generated/`, `gen/`, `*_generated.*`, `*.pb.go`, `*.pb.ts`, `*.g.dart`

#### Rule GEN2 — CLAUDE.md says "edit specs, not generated code"
- **Check**: CLAUDE.md explicitly instructs to edit `.proto`, `.graphql`, `schema.prisma`, `openapi.yaml` — not their outputs.
- **Why**: New Claude users instinctively edit the generated TypeScript. Regeneration overwrites their changes. This is the #1 time-wasting mistake.
- **Severity**: HIGH

#### Rule GEN3 — Regeneration command is documented
- **Check**: CLAUDE.md contains the codegen command (`protoc`, `prisma generate`, `npm run codegen`, etc.).
- **Why**: After editing a spec, Claude needs to regenerate. Without the command, it guesses or skips regeneration, leading to type mismatches.
- **Severity**: HIGH

#### Rule GEN4 — PostToolUse hook auto-regenerates
- **Check**: A hook runs codegen after spec files are edited.
- **Why**: Forgetting to regenerate causes confusing type errors. Automation eliminates this entire class of mistakes.
- **Severity**: MEDIUM
- **Example**:
  ```json
  {
    "hooks": {
      "PostToolUse": [{
        "matcher": "Edit|Write",
        "hooks": [{
          "type": "command",
          "command": "jq -r '.tool_input.file_path' | grep -qE '\\.(proto|graphql|prisma)$' && npm run codegen || true"
        }]
      }]
    }
  }
  ```

#### Rule GEN5 — Spec files are the source of truth for types
- **Check**: Type definitions live in spec files, not duplicated in application code.
- **Why**: Duplicated types = Claude edits one but not the other = type drift. Single source of truth in the spec = Claude always reads the right thing.
- **Severity**: MEDIUM

#### Codegen Detection Heuristic
```
IF *.proto files exist → PROTOBUF
IF *.graphql or schema.graphql exists → GRAPHQL
IF openapi.yaml or swagger.json exists → OPENAPI
IF schema.prisma exists → PRISMA
IF buf.yaml exists → BUF_PROTOBUF
```

---

### 6.10 — Polyglot / Multi-Language Edge Cases

Projects with 2+ languages (e.g., Go backend + TypeScript frontend, Python ML + Rust inference).

#### Rule POLY1 — Each language has its own CLAUDE.md
- **Check**: Each language-specific directory has its own CLAUDE.md with language-specific conventions.
- **Why**: Go conventions (error handling, struct tags) are irrelevant when Claude works in the TypeScript directory. Loading all rules always wastes tokens.
- **Severity**: HIGH

#### Rule POLY2 — Root CLAUDE.md is language-agnostic
- **Check**: Root CLAUDE.md contains only cross-cutting concerns (git workflow, CI, shared conventions). No language-specific rules.
- **Why**: Root CLAUDE.md loads on every request. Language-specific rules in root = wasted tokens 50%+ of the time.
- **Severity**: HIGH

#### Rule POLY3 — .claude/rules/ uses path-scoped language rules
- **Check**: Rules are scoped to language directories (`paths: "backend/**"`, `paths: "frontend/**"`).
- **Why**: A React component rule should never load when Claude edits a Go handler. Path scoping ensures only relevant rules load.
- **Severity**: MEDIUM

#### Rule POLY4 — Each language has independent build/test commands
- **Check**: CLAUDE.md documents separate build/test commands per language (not just one global command).
- **Why**: `make test` that runs everything takes 5 minutes. `cd backend && go test ./pkg/auth` takes 5 seconds. Claude needs the fast path.
- **Severity**: HIGH

#### Rule POLY5 — Shared contracts are in a neutral format
- **Check**: Cross-language API contracts use a neutral format (protobuf, OpenAPI, JSON Schema), not one language's types.
- **Why**: Claude reads the neutral spec to understand the contract, then implements in the target language. Language-specific types force Claude to translate mentally.
- **Severity**: MEDIUM

#### Rule POLY6 — .claudeignore covers ALL language runtimes
- **Check**: .claudeignore includes dependency directories for every language in the project.
- **Why**: A Go+Node project needs both `vendor/` and `node_modules/` ignored. Missing one = half the project's noise leaks through.
- **Severity**: HIGH

#### Polyglot Detection Heuristic
```
IF (go.mod AND package.json) exists → GO_NODE_POLYGLOT
IF (Cargo.toml AND package.json) exists → RUST_NODE_POLYGLOT
IF (requirements.txt AND package.json) exists → PYTHON_NODE_POLYGLOT
IF 2+ different language manifests at same level → POLYGLOT
```

---

### 6.11 — Legacy / Brownfield Edge Cases

Old codebases with technical debt, mixed patterns, and missing tests.

#### Rule LEG1 — CLAUDE.md documents the "mess"
- **Check**: CLAUDE.md honestly describes: known inconsistencies, deprecated patterns, which patterns are "correct" vs "legacy".
- **Why**: Claude sees multiple patterns and picks one. Without guidance on which is correct, it picks the legacy pattern 50% of the time. A brief "error handling: use the pattern in `utils/error.js`, NOT the old try-catch in `handlers/`" saves hours.
- **Severity**: HIGH

#### Rule LEG2 — "Correct" patterns are explicitly identified
- **Check**: CLAUDE.md or `.claude/rules/` identifies the modern/correct pattern for each concern with a file reference.
- **Why**: Legacy codebases have 3+ patterns for the same concern. Claude needs one canonical reference.
- **Severity**: HIGH
- **Example**:
  ```markdown
  # Patterns
  - Error handling: follow `src/services/auth.ts` (NOT `src/handlers/legacy.ts`)
  - Data access: use `src/db/repository.ts` pattern (NOT raw SQL in handlers)
  - API responses: use `src/utils/response.ts` wrapper (NOT manual res.json)
  ```

#### Rule LEG3 — Tests exist for code being modified
- **Check**: At minimum, the specific module being worked on has tests. Full coverage is not required.
- **Why**: In legacy projects, full test coverage is unrealistic. But changes without ANY tests are dangerous. Claude should write a test FIRST, then modify.
- **Severity**: HIGH

#### Rule LEG4 — Dead code is flagged, not hidden
- **Check**: CLAUDE.md lists known dead code directories/files so Claude skips them.
- **Why**: Legacy projects accumulate dead code that wastes Claude's exploration tokens. Explicit flagging prevents Claude from reading and trying to use deprecated functions.
- **Severity**: MEDIUM
- **Example**:
  ```markdown
  # Dead code (do not use or reference)
  - src/old_auth/ — replaced by src/auth/ in 2023
  - src/utils/legacy_*.ts — deprecated helpers
  - src/api/v1/ — replaced by v2, will be removed
  ```

#### Rule LEG5 — Incremental approach is documented
- **Check**: CLAUDE.md instructs to make small, atomic changes with tests — not big-bang refactors.
- **Why**: Legacy codebases have hidden dependencies. A big refactor breaks things in unexpected places. Small changes with tests are safer.
- **Severity**: MEDIUM

#### Rule LEG6 — File size tolerance is higher but documented
- **Check**: If mega-files (>500 lines) exist and can't be split immediately, CLAUDE.md documents them with key function locations.
- **Why**: You can't split a 2000-line legacy file in one session. But you CAN tell Claude "the auth logic is on lines 200-350, ignore the rest."
- **Severity**: MEDIUM
- **Example**:
  ```markdown
  # Large files (known tech debt)
  - src/app.js (1800 lines) — auth: L200-350, routing: L400-600, middleware: L700-900
  - src/db.js (1200 lines) — queries: L100-500, migrations: L600-900
  ```

#### Legacy Detection Heuristic
```
IF no test files exist
   AND average file size > 300 lines
   AND no CLAUDE.md
   AND project age > 2 years (from oldest git commit)
THEN → LEGACY

IF test coverage < 20% (if measurable)
   AND multiple contradictory patterns detected
THEN → BROWNFIELD
```

#### Legacy Scoring Adjustments
- Rule 5.2 (tests exist) severity stays HIGH but scope narrows to "tests exist for modified code"
- Rule 2.1 (mega-files) is relaxed if CLAUDE.md documents line ranges
- New rules LEG1, LEG2 are CRITICAL for legacy — without them Claude is working blind
- Weights shift:
  - Foundation: 25% → **35%** (documentation is the lifeline)
  - Context Efficiency: 25% → **15%** (legacy files are large, accept it)
  - Navigation: 20% → **15%** (structure is what it is)
  - Tooling: 15% → **10%** (less relevant for legacy)
  - Code Quality: 15% → **25%** (tests and patterns matter most for safe changes)

---

### 6.12 — Documentation Site Edge Cases (Docusaurus, MkDocs, Hugo)

Projects that are primarily markdown content with minimal code.

#### Rule DOC1 — Build output is ignored
- **Check**: `build/`, `dist/`, `public/`, `.docusaurus/`, `site/` are in .claudeignore.
- **Why**: Doc sites generate HTML from markdown. The HTML is 5-10x larger than the source and provides no value to Claude.
- **Severity**: HIGH

#### Rule DOC2 — Search indices are ignored
- **Check**: Algolia/Lunr/Pagefind search index files are in .claudeignore.
- **Why**: Search indices are generated JSON blobs that can be 1MB+.
- **Severity**: MEDIUM

#### Rule DOC3 — Markdown source is the focus
- **Check**: CLAUDE.md points to the markdown source directory (not build output).
- **Why**: Claude should edit `docs/*.md`, not `build/*.html`. This seems obvious but generated sites often have confusing directory structures.
- **Severity**: LOW

#### Rule DOC4 — Navigation config is documented
- **Check**: CLAUDE.md explains how to add a new page (which sidebar/nav file to update).
- **Why**: Each doc framework has different nav config (`sidebars.js`, `mkdocs.yml`, `config.toml`). Claude needs to know where to register new pages.
- **Severity**: MEDIUM

#### Rule DOC5 — Template/style files are minimal
- **Check**: Custom CSS/theme overrides are <200 lines.
- **Why**: Doc sites should use framework defaults. Heavy customization wastes Claude tokens on understanding the theme.
- **Severity**: LOW

#### Doc Site Detection Heuristic
```
IF docusaurus.config.* exists → DOCUSAURUS
IF mkdocs.yml exists → MKDOCS
IF hugo.toml OR config.toml with baseURL → HUGO
IF .vuepress/ exists → VUEPRESS
IF docs/ with mostly *.md files AND no significant src/ → DOC_SITE
```

---

### 6.13 — Game Development Edge Cases (Unity, Godot, Bevy)

Game projects mix human-readable scripts with massive binary assets.

#### Rule GAME1 — Binary scene/asset files are ignored
- **Check**: Scene files, models, textures, audio are in .claudeignore.
- **Why**: Binary files like `.unity`, `.prefab`, `.tscn`, `.blend`, `.fbx` can't be read by Claude. They clutter search results and waste tokens on read errors.
- **Severity**: CRITICAL
- **Must ignore**: `*.unity`, `*.prefab`, `*.asset`, `*.tscn`, `*.tres`, `*.blend`, `*.fbx`, `*.png`, `*.wav`, `*.mp3`

#### Rule GAME2 — Editor-generated metadata is ignored
- **Check**: `*.meta` (Unity), `Library/` (Unity), `.godot/` (Godot), `.import/` (Godot) are in .claudeignore.
- **Why**: Unity alone generates a `.meta` file for every single asset. A 500-asset project = 500 extra files in search results.
- **Severity**: HIGH

#### Rule GAME3 — Scripts are the primary target
- **Check**: CLAUDE.md directs to script directories (not scene/prefab paths).
- **Why**: Claude can read and modify C# (Unity), GDScript (Godot), Rust (Bevy). It CANNOT modify scenes, prefabs, or inspector values. CLAUDE.md must point Claude to the code, not the editor files.
- **Severity**: HIGH
- **Example**:
  ```markdown
  # Source code Claude should work with
  - Game logic: Assets/Scripts/
  - Shaders: Assets/Shaders/
  - Editor tools: Assets/Editor/
  # Do NOT reference scene or prefab paths
  ```

#### Rule GAME4 — Inspector/config values are documented in code
- **Check**: Serialized fields have default values and comments explaining tuning.
- **Why**: Claude can't see Unity Inspector values. If `speed = 5f` is set in the Inspector but the code says `float speed;`, Claude doesn't know the runtime value.
- **Severity**: MEDIUM
- **Example**: `[SerializeField] float moveSpeed = 5f; // Tuned for player feel, don't change without playtesting`

#### Rule GAME5 — Test framework exists for game logic
- **Check**: Unit tests exist for non-visual game logic (state machines, inventory, damage calculation).
- **Why**: Game code is often untested because "you need to play it." But pure logic (math, state, rules) is perfectly testable and is where Claude adds most value.
- **Severity**: MEDIUM

#### Game Detection Heuristic
```
IF *.csproj with UnityEngine reference → UNITY
IF project.godot exists → GODOT
IF Cargo.toml with bevy dependency → BEVY
IF Unreal *.uproject exists → UNREAL
```

---

### 6.14 — Project Type Detection: Complete Heuristic

```
PHASE 1 — Check repo structure:
  IF multiple package manifests at different levels
     OR workspace config exists → MONOREPO
  IF single manifest AND <50 files AND depth ≤3 → MICRO_REPO

PHASE 2 — Check domain-specific markers:
  IF pubspec.yaml → FLUTTER
  IF react-native in package.json → REACT_NATIVE
  IF *.xcodeproj AND no cross-platform markers → IOS_NATIVE
  IF android plugin in build.gradle.kts AND no cross-platform → ANDROID_NATIVE
  IF next.config.* → NEXTJS
  IF angular.json → ANGULAR
  IF *.tf files → TERRAFORM
  IF Chart.yaml → HELM
  IF serverless.yml OR SAM template.yaml → SERVERLESS
  IF *.ipynb AND ML frameworks in requirements → ML_PROJECT
  IF *.proto OR schema.prisma OR schema.graphql → CODEGEN_HEAVY
  IF docusaurus.config.* OR mkdocs.yml → DOC_SITE
  IF UnityEngine OR project.godot OR bevy in Cargo.toml → GAME_DEV
  IF manage.py → DJANGO
  IF Gemfile with rails → RAILS
  IF go.mod with cmd/ → GO_SERVICE

PHASE 3 — Check for compound types:
  IF 2+ language manifests → add POLYGLOT flag
  IF no tests AND avg file >300 lines AND age >2 years → add LEGACY flag

RESULT: Primary type + optional flags (e.g., DJANGO + LEGACY + POLYGLOT)
```

---

### 6.15 — Master Summary: How All Project Types Affect Scoring

| Project Type | Foundation | Context | Navigation | Tooling | Code Quality | Key Override |
|-------------|-----------|---------|------------|---------|-------------|-------------|
| **Standard** | 25% | 25% | 20% | 15% | 15% | — |
| **Monorepo** | 30% | 25% | 20% | 15% | 10% | Per-package CLAUDE.md required |
| **Micro-repo** | 20% | 20% | 15% | 15% | 30% | Tests critical, tooling relaxed |
| **Mobile** | 25% | 30% | 15% | 15% | 15% | Platform dirs MUST be ignored |
| **Frontend SPA** | 25% | 30% | 15% | 15% | 15% | Build cache MUST be ignored |
| **Backend API** | 25% | 20% | 20% | 15% | 20% | Migrations, data access docs |
| **IaC** | 30% | 20% | 15% | 20% | 15% | State files = CRITICAL block |
| **Serverless** | 25% | 25% | 15% | 20% | 15% | Small function handlers |
| **Data/ML** | 20% | 35% | 10% | 15% | 20% | Binary artifacts MUST be ignored |
| **Codegen Heavy** | 25% | 30% | 15% | 20% | 10% | Edit specs, not generated code |
| **Polyglot** | 30% | 20% | 20% | 15% | 15% | Per-language CLAUDE.md |
| **Legacy** | 35% | 15% | 15% | 10% | 25% | Document the mess |
| **Doc Site** | 20% | 25% | 20% | 15% | 20% | Markdown source, not HTML |
| **Game Dev** | 25% | 35% | 15% | 10% | 15% | Binary assets MUST be ignored |

---

## Suggestion Priority

When generating suggestions, order by:
1. **Bootstrap** — when score < 60, suggest `claude-native --init` first (one command, ~25 point jump)
2. **Quick wins** — things that take <2 minutes and have high impact (create .claudeignore, add build command to CLAUDE.md)
3. **High impact** — structural changes that significantly improve the score (split mega-files, add tests)
4. **Nice to have** — optimizations that help but aren't critical (add hooks, create skills)

---

## Lessons Learned (from self-scoring exercise)

These are implementation insights discovered by running the scanner on its own codebase (D→A+ journey):

1. **Suggestions should show exact file content** — "Create .claudeignore" is vague. Show the actual patterns tailored to the detected project type.
2. **`.claudeignore` must be respected by ALL rules** — Not just mega-files. Every rule that reads `all_files` should exclude claudeignored files.
3. **Function-length thresholds should be practical** — 50 was too strict; 80 is where functions actually start hurting. Registry functions (lists of `Box::new`) are long but not complex.
4. **Brace-counting breaks on string/char literals** — `"=> {"` and `'{'` contain literal braces. Any code analysis must skip content inside quotes.
5. **Detection order matters** — Domain markers (Flutter, Django) must run before structural checks (micro-repo). A 3-file Flutter project is Flutter, not a micro-repo.
6. **Naming heuristics need skip lists** — ALL_CAPS files, standard names (Makefile, Cargo.toml), single-word lowercase (mod, lib) must be excluded from convention checks.
7. **Stub rules that always pass are worse than no rule** — Use `skip()` for unimplemented checks, not `pass()`. A passing rule claims the project is good.
8. **Test ratio should count functions, not just files** — A file with 30 test functions is worth more than 30 files with 1 test each.
9. **Quick wins are massively impactful** — CLAUDE.md + .claudeignore + settings.json = 28 point jump in 10 minutes. Always prioritize these.
10. **The scanner should eat its own dogfood** — Running the tool on itself reveals the most actionable bugs and threshold issues.
