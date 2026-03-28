# Brainstorm: Competitive Landscape for claude-native
**Date**: 2026-03-29
**Type**: exploration

## Central Question
What tools exist in the market doing similar things to claude-native, and how do we differentiate?

## Mind Map

```
                          ┌─── Direct Competitors (AI-readiness scorers)
                          │    ├── Microsoft AgentRC (707 stars, TypeScript)
                          │    ├── Factory.ai (proprietary SaaS, coined the category)
                          │    ├── AIReady CLI (code-level analysis, TS-only)
                          │    ├── @kodus/agent-readiness (39 checks, open-source)
                          │    └── agent-readiness-score (Python, early stage)
                          │
                          ├─── Claude-Specific Health Checkers
                          │    ├── cc-health-check (20 checks, npm)
                          │    ├── claude-health (Claude Code skill, tw93)
                          │    └── claude-code-health-check (45 checks, Windows)
                          │
  [claude-native]─────────├─── Config Generators (adjacent, not scoring)
                          │    ├── ai-rulez (18-tool config from single YAML)
                          │    ├── @schuettc/claude-code-setup (setup wizard)
                          │    ├── claude-rules (CLAUDE.md generator)
                          │    ├── Agent Rules Builder (web app, 1000+ rules)
                          │    ├── rule-porter (format converter)
                          │    └── @claude-collective/cli (80+ skill modules)
                          │
                          ├─── General Code Health (different domain)
                          │    ├── CodeScene (enterprise, 25 biomarkers)
                          │    ├── SonarQube (industry standard)
                          │    └── code-health-meter (JS/TS metrics)
                          │
                          └─── Standards & Specs
                               ├── AGENTS.md (Linux Foundation, 60K+ projects)
                               └── Damian Galarza's Assessment (consulting service)
```

## Deep Dive: Direct Competitors

### Microsoft AgentRC — The 800-lb gorilla
| Aspect | AgentRC | claude-native |
|--------|---------|---------------|
| Stars | 707 | New |
| Language | TypeScript | Rust |
| Focus | Copilot-centric, multi-agent | Claude Code-specific |
| Distribution | npx | cargo install |
| Scoring | 9 pillars, 5 maturity levels | 5 dimensions, 95 rules, A+-F |
| Generation | AGENTS.md, copilot-instructions.md | CLAUDE.md, .claudeignore, settings.json |
| CI/CD | Context drift monitoring | Exit code 1 if <40 |
| Fix mode | Yes (generates files) | Yes (--fix, --init) |
| Offline | Yes | Yes |
| Backing | Microsoft | Independent |

**Key gap:** AgentRC doesn't check for Claude-specific config: .claudeignore, hooks, MCP, skills, path-scoped rules. It generates AGENTS.md and copilot-instructions.md — not CLAUDE.md.

### Factory.ai — The category creator
- Coined "Agent Readiness" concept
- Proprietary, cloud-only, token-based pricing
- 60+ LLM-grounded criteria, automated PR remediation
- **Key gap:** Not open-source, not local-first, not Claude-specific

### AIReady CLI — Closest in spirit
- 0-100 score with 3 weighted dimensions
- Detects semantic duplicates, context fragmentation
- TypeScript/JavaScript only
- **Key gap:** Code-level analysis only, no project config checks

### @kodus/agent-readiness — Open-source alternative
- 39 checks, 7 pillars, radar chart dashboard
- MIT, runs locally, 10+ languages
- **Key gap:** General agent-readiness, no Claude-specific checks

## Deep Dive: claude-native's Unique Position

### What ONLY claude-native does:
1. **Claude Code ecosystem checks** — CLAUDE.md quality, .claudeignore glob patterns, hooks (PostToolUse/PreToolUse), MCP server configs, path-scoped rules, skills
2. **14 project type detection** with per-type weight adjustment (Flutter gets 30% Context weight, micro-repo gets 30% Code Quality weight)
3. **Real function-length detection** with string/char literal-aware brace counting
4. **Token-cost estimates** in suggestions ("saves ~2000 tokens per search")
5. **--diff mode** showing before/after without making changes
6. **--watch mode** for live re-scoring
7. **Rust** — single binary, no runtime, fast

### What competitors do that we don't:
1. **AGENTS.md support** — the emerging multi-agent standard (AgentRC, ai-rulez)
2. **Multi-AI-tool output** — generate configs for Cursor, Copilot, Gemini (ai-rulez, rule-porter)
3. **Web dashboard** — visual radar charts (Kodus)
4. **Context drift monitoring** — CI/CD integration that catches when docs go stale (AgentRC)
5. **Code-level AI comprehensibility** — semantic duplicate detection, pattern consistency (AIReady)
6. **npx distribution** — zero-install for Node.js ecosystem

## Opportunities Identified

### 1. Add AGENTS.md support (HIGH priority)
The AGENTS.md standard is adopted by 60,000+ projects and backed by OpenAI, Anthropic, Cursor. Add:
- Rule: AGENTS.md exists
- --init generates AGENTS.md alongside CLAUDE.md
- Score bonus for having both

### 2. Multi-AI output mode (MEDIUM priority)
`claude-native --init --format all` generates:
- CLAUDE.md (Claude Code)
- .cursorrules (Cursor)
- .github/copilot-instructions.md (Copilot)
- AGENTS.md (universal)

### 3. npx wrapper (HIGH priority for adoption)
`npx claude-native` — wrap the Rust binary in an npm package for zero-install in the Node.js ecosystem (like @biomejs/biome does).

### 4. Web dashboard / badge (MEDIUM priority)
- `claude-native --badge` generates a shield.io badge: `![Claude Native Score](https://img.shields.io/badge/claude--native-A%2B-green)`
- Web version at claude-native.web.app that accepts GitHub URLs

### 5. Context drift detection (LOW priority for v1)
CI workflow that compares CLAUDE.md against actual project state and flags when docs are stale.

## Discussion Log

### Key Competitive Insights
- **The category is real and validated** — Microsoft, Factory.ai, and 5+ open-source projects confirm market demand
- **No one owns Claude-specific optimization** — Every competitor is either Copilot-centric (AgentRC), proprietary (Factory.ai), or agent-agnostic (Kodus, AIReady)
- **Our unique angle is clear:** the ONLY tool that deeply understands Claude Code's ecosystem (CLAUDE.md + .claudeignore + hooks + MCP + skills + rules + settings.json)
- **Rust is a differentiator** — every competitor uses TypeScript/Python. Rust = fast, single binary, no runtime
- **95 rules is competitive** — AgentRC has 9 pillars, Kodus has 39 checks, AIReady has 3 dimensions. We have the most granular rule set

### Market Positioning
```
                    General ←────────────────→ Claude-Specific

  Enterprise  ▲     Factory.ai    CodeScene
              │
              │     AgentRC
              │
              │     Kodus         cc-health-check
              │
              │     AIReady       claude-health
  Open-Source ▼
                                  ▶ claude-native ◀
                                    (95 rules, Rust,
                                     14 project types,
                                     --init/--fix/--diff/--watch)
```

## Synthesis

### Key Insights
1. **We're in the right category at the right time** — "agent readiness scoring" is an emerging market with Microsoft, Factory.ai, and multiple startups validating demand
2. **Claude-specific depth is our moat** — no competitor checks for .claudeignore patterns, PostToolUse hooks, MCP server configs, path-scoped rules, or skill directories
3. **AGENTS.md is the one gap to close** — it's the universal standard backed by all major AI tools including Anthropic
4. **Distribution gap** — every competitor uses npx for zero-install. We need an npm wrapper or WASM build
5. **The scoring market will consolidate** — tools will either become agent-agnostic (like AGENTS.md) or agent-specific (like claude-native). We should be the definitive Claude Code-specific tool

### Decision Points
- Should we add AGENTS.md support? → **Yes, immediately**
- Should we support multi-AI output (Cursor, Copilot)? → **Yes, but v1.1**
- Should we add npx distribution? → **Yes, high priority for adoption**
- Should we add a web dashboard? → **Later, after CLI is established**
- Should we rename to something more general? → **No — "claude-native" positions us clearly**

### Next Steps
1. **v1.1:** Add AGENTS.md rule + generation
2. **v1.1:** Create npm wrapper package for `npx claude-native`
3. **v1.2:** Multi-AI output (--format cursor, --format copilot, --format all)
4. **v1.2:** Badge generation (`--badge`)
5. **v2.0:** Context drift detection in CI
6. **v2.0:** Code-level AI comprehensibility checks (inspired by AIReady)

### Competitive Summary Table

| Tool | Type | Rules | Claude-Specific | Open Source | Local | Fix Mode |
|------|------|-------|-----------------|-------------|-------|----------|
| **claude-native** | CLI (Rust) | 95 | Deep | MIT | Yes | Yes |
| Microsoft AgentRC | CLI (TS) | 9 pillars | No (Copilot) | MIT | Yes | Yes |
| Factory.ai | SaaS | 60+ | No | No | No | Yes |
| AIReady CLI | CLI (TS) | 3 dims | No | MIT | Yes | No |
| @kodus/agent-readiness | CLI (TS) | 39 | No | MIT | Yes | No |
| cc-health-check | CLI (TS) | 20 | Config only | MIT | Yes | No |
| claude-health | Skill | 6 layers | Config only | MIT | Yes | No |

**Bottom line:** claude-native is the most comprehensive Claude Code-specific project optimization tool in the market, with the most rules (95), the most project types (14), and the most CLI modes (7). The gaps to close are AGENTS.md support and npx distribution.
