# scan/

Project directory scanner. Builds `ProjectContext` used by all rules.
- `builder.rs` — walks filesystem, reads key files, builds context
- `classifiers.rs` — file type detection (test, generated, lock, manifest)
- `file_stats.rs` — line counting and function-length detection
