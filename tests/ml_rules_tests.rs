mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::ml::*;

#[test]
fn model_files_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("requirements.txt", "torch==2.1.0"),
        ("notebook.ipynb", "{}"),
        ("models/best.pth", "binary"),
        (".claudeignore", "*.pth\n*.pkl\n*.h5\n"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = ModelFilesIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn experiment_workflow_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("requirements.txt", "torch==2.1.0"),
        ("notebook.ipynb", "{}"),
        ("src/train.py", "import torch"),
        ("CLAUDE.md", "# ML\nBuild: `pip install -e .`\nTest: `pytest`\nTrain: `python train.py`\nEvaluate: check metrics in wandb."),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = ExperimentWorkflowDocumented.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn requirements_pinned_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("requirements.txt", "torch==2.1.0\nnumpy==1.24.0\npandas==2.0.0"),
        ("notebook.ipynb", "{}"),
        ("src/train.py", "import torch"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = RequirementsPinned.check(&ctx);
    assert!(result.status.is_pass());
}
