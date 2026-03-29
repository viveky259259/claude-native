# src/

Root source directory. Key entry points:
- `main.rs` — CLI entry, flag dispatch
- `lib.rs` — public module re-exports
- `cli.rs` — clap argument definitions
- `init.rs` / `fix.rs` — file generation and auto-repair
- `scoring/` — score calculation and grade assignment
