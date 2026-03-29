# rules/

Rule implementations for the 5 scoring dimensions.
Every rule struct implements the `Rule` trait from `mod.rs`.
Use `self.pass()`, `self.fail()`, `self.warn()`, `self.skip()` helpers.

Files: foundation, context, navigation, tooling, quality (+ `_extra` splits).
Project-specific rules are in `project_specific/`.
