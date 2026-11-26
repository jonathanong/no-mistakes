use super::{BatchCheckFactRequest, CheckFactPlan, FactDemand};
use std::collections::BTreeMap;
use std::path::PathBuf;

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
            forbidden_call_arguments: request.plan.forbidden_call_arguments.clone(),
            graph: request.plan.graph,
            call_site_paths: request.plan.call_site_paths.clone(),
            nested_control_paths: request.plan.nested_control_paths.clone(),
            vitest_default_exclude_paths: request.plan.vitest_default_exclude_paths.clone(),
            csp_allowed_origins_paths: request.plan.csp_allowed_origins_paths.clone(),
            test_reliability_paths: request.plan.test_reliability_paths.clone(),
            playwright_fixture_support: request.plan.playwright_fixture_support,
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
        }
        // Independently configured sources remain addressable without joining
        // the primary or graph universe. Existing placements keep their plan.
        let mut supplemental = request
            .plan
            .vitest_default_exclude_paths
            .iter()
            .chain(request.plan.csp_allowed_origins_paths.iter())
            .chain(request.plan.nested_control_paths.iter())
            .chain(request.plan.test_reliability_paths.keys())
            .filter(|path| !plans.contains_key(path.as_path()))
            .cloned()
            .collect::<Vec<_>>();
        if let Some(playwright) = &request.playwright {
            for path in playwright.config_files() {
                // Staged collection already owns these ordinary config programs.
                plans.remove(&normalize(path));
            }
        }
        if let Some(runner) = &request.plan.integration_runner_configs {
            for path in runner.paths() {
                plans.remove(&normalize(path));
            }
        }
        let is_owned_config = |path: &PathBuf| {
            request
                .playwright
                .as_ref()
                .is_some_and(|playwright| playwright.config_files().contains(path))
                || request
                    .plan
                    .integration_runner_configs
                    .as_ref()
                    .is_some_and(|runner| runner.contains(path))
        };
        supplemental.retain(|path| !is_owned_config(path));
        if !supplemental.is_empty() {
            let supplemental_plan = CheckFactPlan {
                nested_control_paths: request.plan.nested_control_paths.clone(),
                vitest_default_exclude_paths: request.plan.vitest_default_exclude_paths.clone(),
                csp_allowed_origins_paths: request.plan.csp_allowed_origins_paths.clone(),
                test_reliability_paths: request.plan.test_reliability_paths.clone(),
                playwright_fixture_support: request.plan.playwright_fixture_support,
                ..CheckFactPlan::default()
            };
            plans.extend(
                supplemental
                    .into_iter()
                    .map(|path| (path, supplemental_plan.clone())),
            );
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
