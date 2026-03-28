mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::backend::*;

#[test]
fn migration_history_passes_few_migrations() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("manage.py", "#!/usr/bin/env python"),
        ("requirements.txt", "django==4.2"),
        ("migrations/0001_initial.py", "# migration"),
        ("migrations/0002_users.py", "# migration"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = MigrationHistoryManageable.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn virtual_envs_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("manage.py", "#!/usr/bin/env python"),
        ("requirements.txt", "django==4.2"),
        (".claudeignore", ".venv/\n__pycache__/\n"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = VirtualEnvsIgnored.check(&ctx);
    assert!(result.status.is_pass());
}
