# Competitive Analysis — Features We're Missing

## All features competitors have that we don't

### From Microsoft AgentRC
1. AGENTS.md generation (universal AI agent standard, 60K+ projects)
2. Copilot-instructions.md generation
3. Context drift detection in CI/CD (flags when docs go stale vs actual code)
4. Batch mode — scan multiple repos at once
5. Eval mode — test if AI agents produce correct results using generated context
6. 5-level maturity model (not just A-F grades)
7. VS Code extension

### From Factory.ai
8. Automated remediation via PRs (not just suggestions, actual PRs)
9. Observability pillar (logging, monitoring, tracing checks)
10. Security & governance pillar (SAST, secrets scanning integration)
11. LLM-grounded criteria (use an LLM to evaluate code quality, not just heuristics)
12. Team/org dashboards with historical trends

### From AIReady CLI
13. Semantic duplicate detection (finds copy-pasted code that confuses AI)
14. Context fragmentation analysis (detects scattered related logic)
15. Pattern inconsistency detection (finds conflicting implementations)
16. MCP server mode (run as MCP tool inside IDE)
17. AI Signal Clarity scoring (how clearly code communicates intent to AI)

### From @kodus/agent-readiness
18. Interactive web dashboard with radar charts
19. Historical trend tracking across runs
20. Per-pillar breakdown with actionable remediation steps
21. 10+ language support with language-specific checks

### From cc-health-check / claude-health
22. Subagent configuration audit (checks .claude/agents/ setup)
23. Verifier configuration audit
24. Recovery/rollback safety checks
25. Autonomy calibration (how much unsupervised work is safe)

### From ai-rulez
26. Single-source config → multi-tool output (Cursor, Copilot, Gemini, Windsurf, etc.)
27. Config sync watching (auto-regenerate on source change)

### From @claude-collective/cli
28. Skill marketplace browsing and installation
29. Stack templates (pre-built config bundles for common project types)

### From Agent Rules Builder
30. 1000+ pre-built rule templates browsable by framework/language
31. Web UI for building rules visually

### From CodeScene
32. Code health biomarkers (Cyclomatic Complexity, Coupling, Cohesion)
33. Hotspot analysis (most-changed + lowest-quality files)
34. Developer knowledge mapping (bus factor per module)

### From General Ecosystem
35. npx distribution (zero-install for Node.js ecosystem)
36. GitHub Action published to marketplace
37. Badge/shield generation for README
38. SARIF output for GitHub Code Scanning integration
39. Pre-commit hook integration
40. Config file for customizing thresholds (.claude-native.yml)

---

## Filtered: What makes sense for claude-native

### YES — High value, fits our mission

| # | Feature | Why it fits | Priority | Effort |
|---|---------|-------------|----------|--------|
| 1 | **AGENTS.md generation** | Universal standard backed by Anthropic. Our --init should generate it. | v1.1 | 2h |
| 3 | **Context drift detection** | CLAUDE.md says "Build: npm test" but package.json has no test script = stale docs. Detectable with rules. | v1.2 | 4h |
| 13 | **Semantic duplicate detection** | Duplicated code wastes tokens — Claude reads both copies. Directly impacts cost. | v2.0 | 8h |
| 14 | **Context fragmentation** | Scattered related logic = more files Claude reads = higher cost. Core to our mission. | v2.0 | 8h |
| 16 | **MCP server mode** | Run claude-native as MCP tool inside Claude Code. `claude-native score` from within Claude. | v1.2 | 4h |
| 22 | **Subagent config audit** | .claude/agents/ is part of Claude Code ecosystem. We should check it. | v1.1 | 1h |
| 35 | **npx distribution** | Every competitor uses npx. We need it for adoption. Wrap Rust binary in npm. | v1.1 | 3h |
| 36 | **GitHub Action** | `uses: viveky259259/claude-native@v1` in workflows. CI is our #1 use case. | v1.1 | 2h |
| 37 | **Badge generation** | `claude-native --badge` → shield.io URL. Social proof in READMEs. | v1.1 | 1h |
| 40 | **Config file** | `.claude-native.yml` for customizing thresholds, disabling rules, setting project type override. | v1.1 | 3h |

### MAYBE — Useful but not core

| # | Feature | Why maybe | When |
|---|---------|-----------|------|
| 8 | **Auto-PR remediation** | Useful for CI but complex. --fix already handles local. PR creation is scope creep. | v2.0+ |
| 15 | **Pattern inconsistency** | Rule 5.4 does basic check. Deep analysis needs AST parsing per language. | v2.0 |
| 18 | **Web dashboard** | Nice for teams but we're CLI-first. Could be a separate project. | v2.0+ |
| 19 | **Historical trends** | Requires persistent storage. Could output to a JSON file that a separate tool visualizes. | v2.0 |
| 21 | **Language-specific checks** | We detect 13 languages. Language-specific rules (Rust borrow checker, Python type hints) are deep work. | v2.0 |
| 26 | **Multi-tool config output** | --init generates CLAUDE.md. Also generating .cursorrules is nice but dilutes focus. | v1.2 |
| 38 | **SARIF output** | GitHub Code Scanning integration. Niche but valuable for enterprise. | v2.0 |
| 39 | **Pre-commit hook** | `claude-native` as pre-commit hook to block commits that lower score. | v1.2 |

### NO — Doesn't fit

| # | Feature | Why not |
|---|---------|---------|
| 2 | Copilot-instructions.md | We're Claude-specific. Let ai-rulez handle multi-tool. |
| 5 | Eval mode (test AI output) | Different product. We score projects, not AI responses. |
| 7 | VS Code extension | We're CLI-first. IDE extension is a separate product. |
| 9 | Observability pillar | Monitoring/tracing is ops, not AI-readiness. |
| 10 | Security governance | SonarQube/Snyk do this better. Not our lane. |
| 11 | LLM-grounded criteria | Requires API calls = cost + latency + privacy concerns. Our offline-first approach is a feature. |
| 12 | Team dashboards | Enterprise feature. We're a developer tool. |
| 24 | Autonomy calibration | Too opinionated. Users decide autonomy level. |
| 25 | Recovery/rollback | Ops concern, not project structure. |
| 28 | Skill marketplace | @claude-collective already does this well. Complementary, not competitive. |
| 29 | Stack templates | Same — @claude-collective's territory. |
| 30 | 1000+ rule templates | Web UI is a different product. |
| 31 | Visual rule builder | Same. |
| 32 | Code biomarkers | CodeScene/SonarQube territory. We focus on AI-readiness, not general code health. |
| 33 | Hotspot analysis | Requires git history analysis. Different tool. |
| 34 | Knowledge mapping | Org-level feature, not project-level. |
| 27 | Config sync watching | --watch already re-scores. Auto-regenerating configs is dangerous. |

---

## v1.1 Roadmap (filtered, prioritized)

```
Must-have:
  1. AGENTS.md generation in --init                    (2h)
  2. npx distribution (npm wrapper for Rust binary)    (3h)
  3. .claude-native.yml config file                    (3h)
  4. Badge generation (--badge)                        (1h)
  5. GitHub Action for marketplace                     (2h)
  6. Subagent config audit rule                        (1h)

Should-have:
  7. MCP server mode                                   (4h)
  8. Context drift detection rules                     (4h)
  9. Pre-commit hook integration                       (2h)

v2.0:
  10. Semantic duplicate detection
  11. Context fragmentation analysis
  12. Multi-tool config output
  13. Historical trend tracking
  14. SARIF output
```
