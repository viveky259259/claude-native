---
paths:
  - "src/rules/**"
  - "src/rules/project_specific/**"
---

# Rule Implementation Guidelines

- Every rule struct implements the `Rule` trait from `src/rules/mod.rs`
- Use helper methods: `self.pass()`, `self.fail(reason, suggestion)`, `self.warn(reason, suggestion)`, `self.skip()`
- Project-specific rules must override `applies_to()` to check `ProjectType`
- Every `fail()` and `warn()` MUST include a `Suggestion` with priority, title, description, effort
- Rule IDs must match GOLDEN_RULES.md (e.g., "1.1", "M3", "MOB2", "μ1")
- Severity levels: Critical (caps dimension at 30), High (-20), Medium (-10), Low (-5)
