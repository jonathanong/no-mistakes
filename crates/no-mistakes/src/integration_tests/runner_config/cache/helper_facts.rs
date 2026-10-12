use super::*;

pub(super) fn collect_helper_facts(path: &Path, program: &oxc_ast::ast::Program<'_>, source: &str) {
    let path = crate::codebase::ts_resolver::normalize_path(path);
    let request = REQUEST_CACHES.with(|caches| {
        let mut caches = caches.borrow_mut();
        let request = caches.last_mut()?;
        if !request.helper_fact_paths.insert(path.clone()) {
            return None;
        }
        request.fact_plan.as_ref().and_then(|plan| {
            let context_index = if plan.primary_files.contains(&path) {
                0
            } else if plan.graph_files.contains(&path) {
                1
            } else {
                return None;
            };
            Some((
                plan.clone(),
                std::sync::Arc::clone(&request.fact_contexts.as_ref()?[context_index]),
                context_index,
            ))
        })
    });
    let Some((request, context, context_index)) = request else {
        return;
    };
    let (plan, playwright) = if context_index == 0 {
        (&request.primary_plan, request.playwright.as_ref())
    } else {
        (&request.graph_plan, request.playwright.as_ref())
    };
    let graph_only_plan = plan
        .integration_runner_configs
        .as_ref()
        .is_some_and(|configs| configs.contains(&path))
        .then(|| {
            // Project discovery already consumes the runner-config syntax from
            // this exact Program. Collect the ordinary graph facts now while
            // suppressing only the duplicate runner-config projection.
            let mut graph_only = plan.clone();
            graph_only.integration_runner_configs = None;
            graph_only
        });
    let plan = graph_only_plan.as_ref().unwrap_or(plan);
    let facts = crate::codebase::check_facts::collect_file_facts_from_program_with_context(
        &request.root,
        &path,
        plan,
        playwright,
        source,
        program,
        None,
        &context,
    );
    REQUEST_CACHES.with(|caches| {
        if let Some(cache) = caches.borrow_mut().last_mut() {
            cache.helper_facts.insert(path, facts);
        }
    });
}
