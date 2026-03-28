# Scan Project

Run claude-native on a target directory and report the results.

## Steps

1. Build: `cargo build`
2. Run: `cargo run -- <target_path>`
3. If JSON needed: `cargo run -- <target_path> -o json`
4. Report the score, grade, and top 5 suggestions
