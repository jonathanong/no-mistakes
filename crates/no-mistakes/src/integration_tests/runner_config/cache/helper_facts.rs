use super::*;

pub(super) fn collect_helper_facts(path: &Path, program: &oxc_ast::ast::Program<'_>, source: &str) {
    let path = crate::codebase::ts_resolver::normalize_path(path);
    let request = REQUEST_CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let request = caches.last_mut()?;
        if !request.helper_fact_paths.insert(path.clone()) {
            return None;
        }
        request.fact_plan.clone()
    });
    let Some(request) = request else {
        return;
    };
    let (plan, playwright) = if request.primary_files.contains(&path) {
        (&request.primary_plan, request.playwright.as_ref())
    } else if request.graph_files.contains(&path) {
        (&request.graph_plan, request.playwright.as_ref())
    } else {
        return;
    };
    if plan
        .integration_runner_configs
        .as_ref()
        .is_some_and(|configs| configs.contains(&path))
    {
        return;
    }
    let facts = crate::codebase::check_facts::collect_file_facts_from_program(
        &request.root,
        &path,
        plan,
        playwright,
        source,
        program,
    );
    REQUEST_CACHES.with(|caches| {
        if let Some(cache) = caches.borrow_mut().last_mut() {
            cache.helper_facts.insert(path, facts);
        }
    });
}
