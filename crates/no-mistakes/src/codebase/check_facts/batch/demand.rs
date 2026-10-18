use super::*;
use std::collections::BTreeMap;

pub(super) fn demands_by_path(
    requests: &[BatchCheckFactRequest],
) -> BTreeMap<PathBuf, Vec<FactDemand>> {
    let mut demands = BTreeMap::<PathBuf, Vec<FactDemand>>::new();
    for (request_id, request) in requests
        .iter()
        .enumerate()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
    {
        let graph_only = super::super::graph_only_files(&request.files, &request.graph_files);
        let graph_plan = CheckFactPlan {
            graph: request.plan.graph,
            graph_context: request.plan.graph_context.clone(),
            integration_runner_configs: request.plan.integration_runner_configs.clone(),
            ..CheckFactPlan::default()
        };
        let mut plans = BTreeMap::new();
        for path in &request.files {
            plans.insert(normalize(path), request.plan.clone());
        }
        for path in request.plan.production_import_paths.iter() {
            plans
                .entry(normalize(path))
                .or_insert_with(|| request.plan.clone());
        }
        for path in graph_only {
            let path = normalize(&path);
            let mut plan = graph_plan.clone();
            if request.plan.legacy_symbol_paths.contains(&path) {
                // An explicit symbols target can be visible to the graph while
                // remaining outside the configured check file scope.
                plan.symbols = request.plan.symbols;
                plan.legacy_symbol_paths.insert(path.clone());
            }
            plans.entry(path).or_insert(plan);
        }
        if let Some(playwright) = &request.playwright {
            for path in playwright.paths() {
                plans.entry(normalize(path)).or_insert(graph_plan.clone());
            }
            for path in playwright.source_files().iter() {
                let plan = plans.entry(normalize(path)).or_insert(graph_plan.clone());
                // Staged Playwright decides whether source imports are needed
                // after test facts exist. Collect that request-local superset
                // in the same parse; map assembly retains the exact graph plan.
                plan.graph.imports = true;
            }
            for path in playwright.config_files() {
                let path = normalize(path);
                if plans.contains_key(&path) {
                    // Staged collection derives the exact union fact from the
                    // already-cached config AST, so a Rayon demand would only
                    // force the same config through a second parser cache.
                    plans.remove(&path);
                }
            }
        }
        if let Some(runner) = &request.plan.integration_runner_configs {
            for path in runner.paths() {
                plans.remove(&normalize(path));
            }
        }
        for (path, plan) in plans {
            if crate::codebase::dependencies::extract::is_indexable(&path)
                || plan.production_import_paths.contains(&path)
                || plan.storybook && super::super::is_mdx_file(&path)
            {
                demands.entry(path).or_default().push(FactDemand {
                    request: request_id,
                    plan,
                });
            }
        }
    }
    demands
}

fn normalize(path: &std::path::Path) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(path)
}
