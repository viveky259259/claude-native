# claude-native

Scan any project and score how **Claude Native** it is — optimized for lower token cost and higher performance with Claude Code.

## Install

```bash
cargo install --path .
```

## Usage

```bash
# Scan current directory
claude-native

# Scan a specific project
claude-native /path/to/project

# JSON output (for CI)
claude-native . -o json

# Exit code 1 if score < 40 (fail CI on "Claude Hostile" projects)
claude-native . && echo "PASS" || echo "FAIL"
```

## What It Does

1. **Scans** your project directory
2. **Detects** the project type (14 types: Flutter, Next.js, Django, Terraform, etc.)
3. **Runs 94 rules** across 5 dimensions:
   - Foundation — CLAUDE.md, .claudeignore, .claude/ setup
   - Context Efficiency — file sizes, token waste reduction
   - Navigation — structure, naming, entry points
   - Tooling — MCP, hooks, skills, permissions
   - Code Quality — types, tests, consistency
4. **Scores** 0-100 with grade A+ through F
5. **Suggests** fixes prioritized as Quick Wins → High Impact → Nice to Have

## Example Output

```
╔══════════════════════════════════════════════════════════╗
║  Claude Native Score                                    ║
╠══════════════════════════════════════════════════════════╣
║  Project Type: Backend (Django)                         ║
║  Score: 72/100  Grade: B                                ║
║  Claude Friendly — good foundation, notable gaps        ║
╚══════════════════════════════════════════════════════════╝
```

## Supported Project Types

| Type | Detection Signal |
|------|-----------------|
| Monorepo | pnpm-workspace.yaml, nx.json, Cargo workspace |
| Micro-repo | Single manifest, <50 files |
| Flutter | pubspec.yaml |
| React Native | react-native in package.json |
| Next.js | next.config.* |
| Django | manage.py |
| Express | express in package.json |
| Go Service | go.mod + cmd/ |
| Terraform | *.tf files |
| Serverless | serverless.yml, wrangler.toml |
| ML/Data Science | .ipynb + torch/tensorflow |
| Codegen Heavy | .proto/.graphql + generated/ |
| Doc Site | docusaurus.config.*, mkdocs.yml |
| Game Dev | project.godot, Unity scenes, bevy |

Plus compound flags: **Polyglot** (2+ languages) and **Legacy** (no tests, large files, no docs).

## License

MIT
