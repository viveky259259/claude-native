# project_specific/

Rules that only fire for specific project types (monorepo, mobile, backend, etc.).
Each file = one project type. Every rule overrides `applies_to()` to filter by `ProjectType`.
Registered in `mod.rs` via `project_specific_rules()`.
