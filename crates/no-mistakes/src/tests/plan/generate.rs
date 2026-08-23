pub fn generate_plan(args: &PlanArgs) -> Result<TestPlan> {
    no_mistakes::ast::with_request_parse_cache(|| {
        let prepared = super::prepared_plan::PreparedTestPlanRequest::prepare(args)?;
        generate_plan_with_prepared(prepared.args(), &prepared, None)
    })
}
