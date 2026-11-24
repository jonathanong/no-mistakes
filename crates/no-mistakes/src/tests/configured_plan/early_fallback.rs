#[expect(
    clippy::too_many_arguments,
    reason = "The fallback request uses the selected policy and ordered trigger inputs."
)]
fn early_fallback_request<'a>(
    args: &PlanArgs,
    framework: TestFramework,
    env: &no_mistakes::config::v2::schema::TestPlanEnvironment,
    global_config_fallback: bool,
    changed_files: &'a [PathBuf],
    forced_fallback: Option<&'a (String, PathBuf)>,
    dependency_fallback: Option<&'a (String, PathBuf)>,
    (limit, has_limit): (usize, bool),
) -> Option<FallbackRequest<'a>> {
    if let Some((reason, trigger_file)) = forced_fallback.filter(|_| global_config_fallback) {
        return Some(FallbackRequest {
            group_type: "global",
            via: "global configuration",
            changed_file: Some(trigger_file),
            limit,
            has_limit,
            reason: reason.clone(),
        });
    }
    if env.all {
        return Some(FallbackRequest {
            group_type: "all",
            via: "all",
            changed_file: changed_files.first(),
            limit,
            has_limit,
            reason: format!(
                "{} test plan environment `{}` runs all tests",
                framework_name(framework),
                args.environment
            ),
        });
    }
    dependency_fallback.map(|(reason, trigger_file)| FallbackRequest {
        group_type: "dependencies",
        via: "dependency configuration",
        changed_file: Some(trigger_file),
        limit,
        has_limit,
        reason: reason.clone(),
    })
}
