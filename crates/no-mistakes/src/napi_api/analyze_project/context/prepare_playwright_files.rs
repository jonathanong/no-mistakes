fn merge_playwright_fact_files(
    check: Option<&SharedCheckContext>,
    playwright: &HashMap<String, PreparedPlaywrightView>,
    configs: &std::collections::HashSet<PathBuf>,
    files: &mut Vec<PathBuf>,
) -> Option<crate::codebase::check_facts::PlaywrightFactPlan> {
    let mut playwright_fact_plan = check.and_then(SharedCheckContext::playwright_fact_plan);
    for view in playwright.values() {
        match playwright_fact_plan.as_mut() {
            Some(plan) => plan.include(view.fact_plan.clone()),
            None => playwright_fact_plan = Some(view.fact_plan.clone()),
        }
    }
    if let Some(plan) = &playwright_fact_plan {
        files.extend(
            plan.files()
                .iter()
                .filter(|path| !configs.contains(*path))
                .map(|path| crate::codebase::ts_resolver::normalize_path(path))
                .collect::<Vec<_>>(),
        );
        files.sort();
        files.dedup();
    }
    playwright_fact_plan
}
