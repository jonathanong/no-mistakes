use std::collections::BTreeMap;
// Instrumentation-only historical complete builder, retaining inline owned plans.
use super::demands::normalize;
use super::*;
use crate::codebase::check_facts::{graph_only_files, is_mdx_file};

pub(super) struct OwnedFactDemand {
    pub(super) request: usize,
    pub(super) plan: CheckFactPlan,
}

pub(super) fn demands_by_path<const COUNT: bool>(
    requests: &[BatchCheckFactRequest],
    clones: &mut usize,
) -> BTreeMap<PathBuf, Vec<OwnedFactDemand>> {
    let mut demands = BTreeMap::<PathBuf, Vec<OwnedFactDemand>>::new();
    for (request_id, request) in requests
        .iter()
        .enumerate()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
    {
        let graph_only = graph_only_files(&request.files, &request.graph_files);
        let graph_plan = CheckFactPlan {
            forbidden_call_arguments: request.plan.forbidden_call_arguments.clone(),
            graph: request.plan.graph,
            call_site_paths: request.plan.call_site_paths.clone(),
            graph_context: request.plan.graph_context.clone(),
            integration_runner_configs: request.plan.integration_runner_configs.clone(),
            ..CheckFactPlan::default()
        };
        let mut plans = BTreeMap::new();
        for path in &request.files {
            plans.insert(
                normalize(path),
                super::plan::clone_plan::<COUNT>(&request.plan, clones),
            );
        }
        for path in request.plan.production_import_paths.iter() {
            plans
                .entry(normalize(path))
                .or_insert_with(|| super::plan::clone_plan::<COUNT>(&request.plan, clones));
        }
        for path in graph_only {
            let path = normalize(&path);
            let mut plan = super::plan::clone_plan::<COUNT>(&graph_plan, clones);
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
                plans
                    .entry(normalize(path))
                    .or_insert(super::plan::clone_plan::<COUNT>(&graph_plan, clones));
            }
            for path in playwright.source_files().iter() {
                let plan = plans
                    .entry(normalize(path))
                    .or_insert(super::plan::clone_plan::<COUNT>(&graph_plan, clones));
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
                || plan.storybook && is_mdx_file(&path)
            {
                demands.entry(path).or_default().push(OwnedFactDemand {
                    request: request_id,
                    plan,
                });
            }
        }
    }
    demands
}
