use crate::tests::comment::render_markdown_plan;
use crate::tests::{PlanFormat, TestPlan};
use anyhow::Result;
use std::fmt::Write;

pub(crate) fn render(plan: &TestPlan, format: PlanFormat, command_name: &str) -> Result<String> {
    let mut output = String::new();
    match format {
        PlanFormat::Json => writeln!(output, "{}", serde_json::to_string_pretty(plan)?)?,
        PlanFormat::Paths => {
            for test in &plan.selected_tests {
                writeln!(output, "{}", test.test_file)?;
            }
        }
        PlanFormat::Commands => {
            super::targets::ensure_plan_commands_available(plan, command_name)?;
            for command in super::targets::commands_for_plan(plan) {
                writeln!(output, "{command}")?;
            }
        }
        PlanFormat::Markdown | PlanFormat::Md => {
            writeln!(output, "{}", render_markdown_plan(plan))?;
        }
    }
    Ok(output)
}
