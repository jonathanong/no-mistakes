use super::{generate_plan_with_prepared_inner, PlanArgs, Result, TestPlan};

pub(crate) fn generate_plan_with_prepared(
    args: &PlanArgs,
    prepared: &crate::tests::prepared_plan::PreparedTestPlanRequest,
    timing: Option<&mut crate::impacted_checks::timing::TimingTracker>,
) -> Result<TestPlan> {
    let mut plan = generate_plan_with_prepared_inner(args, prepared, timing)?;
    plan.changed_files = prepared.changed_file_inventory();
    Ok(plan)
}
