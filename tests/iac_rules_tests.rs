mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::iac::*;

#[test]
fn provider_cache_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("main.tf", "resource \"aws_instance\" \"web\" {}"),
        (".claudeignore", ".terraform/\n"),
        (".terraform/providers/dummy.txt", "cached"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = ProviderCacheIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn plan_output_filtered_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("main.tf", "resource \"aws_instance\" \"web\" {}"),
        ("CLAUDE.md", "# IaC\nBuild: `terraform plan`\nTest: `terraform validate`\nFilter plan output to resource changes only."),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = PlanOutputFiltered.check(&ctx);
    assert!(result.status.is_pass());
}
