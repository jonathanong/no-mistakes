use super::{generate_plan_with_prepared_inner, PlanArgs, Result, TestPlan};
use crate::codebase::rules::path_filter::GlobMatcher;

pub(crate) fn generate_plan_with_prepared(
    args: &PlanArgs,
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
    timing: Option<&mut crate::impacted_checks::timing::TimingTracker>,
) -> Result<TestPlan> {
    let mut plan = generate_plan_with_prepared_inner(args, prepared, timing)?;
    plan.changed_files = prepared.changed_file_inventory();
    retain_include_glob(&mut plan, &args.include_glob)?;
    plan.finish(args.include_comment, &prepared.config.tests.swift.packages);
    Ok(plan)
}

fn retain_include_glob(plan: &mut TestPlan, patterns: &[String]) -> Result<()> {
    if patterns.is_empty() {
        return Ok(());
    }
    let matcher = GlobMatcher::new(patterns, "includeGlob")?;
    plan.selected_tests
        .retain(|test| matcher.is_match(&test.test_file));
    for group in &mut plan.groups {
        group.selected.retain(|file| matcher.is_match(file));
    }
    Ok(())
}
