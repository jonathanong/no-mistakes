use crate::tests::plan::TestPlan;
use crate::tests::CommentArgs;
use anyhow::{Context, Result};
use std::fs;
use std::process::ExitCode;

pub(crate) fn run(args: CommentArgs) -> Result<ExitCode> {
    let content = fs::read_to_string(&args.plan)
        .with_context(|| format!("Failed to read plan from {}", args.plan.display()))?;
    let plan: TestPlan = serde_json::from_str(&content)
        .context("Failed to parse plan JSON. Make sure it is a valid output of `tests plan`.")?;

    let markdown = render_markdown_plan(&plan);

    if let Some(ref out_path) = args.out {
        fs::write(out_path, &markdown).with_context(|| {
            format!("Failed to write markdown comment to {}", out_path.display())
        })?;
    } else {
        println!("{}", markdown);
    }

    Ok(ExitCode::SUCCESS)
}

pub fn render_markdown_plan(plan: &TestPlan) -> String {
    let mut out = String::new();
    out.push_str("# 🧪 Test Impact Analysis\n\n");

    if plan.fallback_triggered {
        out.push_str("⚠️ **Fallback Triggered**: Running all tests in the workspace.\n");
        if let Some(ref reason) = plan.fallback_reason {
            out.push_str(&format!("- **Reason**: {}\n", reason));
        }
        out.push('\n');
    }

    out.push_str(&format!(
        "## Selected Tests (Total: {})\n\n",
        plan.selected_tests.len()
    ));
    if plan.selected_tests.is_empty() {
        out.push_str("No tests selected.\n\n");
    } else {
        out.push_str("| Test File | Confidence | Reason / Impact Chain |\n");
        out.push_str("| --- | --- | --- |\n");
        for test in &plan.selected_tests {
            let mut reason_desc = String::new();
            for (i, r) in test.reasons.iter().enumerate() {
                if i > 0 {
                    reason_desc.push_str("<br><br>");
                }
                reason_desc.push_str(&format!("Connected to `{}` via:<br>", r.changed_file));
                let chain = r.path.join(" ➔ ");
                reason_desc.push_str(&format!("`{}`", chain));
            }
            out.push_str(&format!(
                "| `{}` | {} | {} |\n",
                test.test_file,
                test.confidence.display_emoji(),
                reason_desc
            ));
        }
        out.push('\n');
    }

    if !plan.warnings.is_empty() {
        out.push_str(&format!("## Warnings (Total: {})\n\n", plan.warnings.len()));
        for w in &plan.warnings {
            out.push_str(&format!(
                "- ⚠️ **{}**: {} (`{}`)\n",
                w.r#type, w.message, w.file
            ));
        }
        out.push('\n');
    }

    out
}
