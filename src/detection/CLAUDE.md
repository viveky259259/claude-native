# detection/

Project type detection. 3-phase heuristic:
1. Structure check (monorepo via workspace configs)
2. Domain markers (Flutter, Django, Terraform, etc.) — in `domain.rs`
3. Micro-repo fallback (single manifest, <50 files)

`signals.rs` + `signals_backend.rs` contain the individual detection functions.
