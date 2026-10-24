use super::plan::DemandPlan;
use super::{BatchCheckFactRequest, CheckFactPlan, FactDemand};
use crate::codebase::check_facts::{graph_only_files, is_mdx_file};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(super) fn graph_plans(requests: &[BatchCheckFactRequest]) -> Vec<Option<CheckFactPlan>> {
    std::iter::repeat_with(|| None)
        .take(requests.len())
        .collect()
}

pub(super) fn demands_by_path<'a, const COUNT: bool>(
    requests: &'a [BatchCheckFactRequest],
    graph_plans: &'a mut [Option<CheckFactPlan>],
    clones: &mut usize,
) -> BTreeMap<PathBuf, Vec<FactDemand<'a>>> {
    let mut demands = BTreeMap::<PathBuf, Vec<FactDemand<'a>>>::new();
    for ((request_id, request), slot) in requests
        .iter()
        .enumerate()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
        .zip(graph_plans.iter_mut())
    {
        let graph_only = graph_only_files(&request.files, &request.graph_files);
        // Construct only after the historical per-request deadline guard.
        let graph_plan: &'a CheckFactPlan = slot.insert(CheckFactPlan {
            forbidden_call_arguments: request.plan.forbidden_call_arguments.clone(),
            graph: request.plan.graph,
            call_site_paths: request.plan.call_site_paths.clone(),
            graph_context: request.plan.graph_context.clone(),
            integration_runner_configs: request.plan.integration_runner_configs.clone(),
            ..CheckFactPlan::default()
        });
        let mut plans = BTreeMap::new();
        for path in &request.files {
            plans.insert(normalize(path), DemandPlan::borrowed(&request.plan));
        }
        for path in request.plan.production_import_paths.iter() {
            plans
                .entry(normalize(path))
                .or_insert_with(|| DemandPlan::borrowed(&request.plan));
        }
        for path in graph_only {
            let path = normalize(&path);
            plans.entry(path.clone()).or_insert_with(|| {
                let mut plan = DemandPlan::borrowed(graph_plan);
                if request.plan.legacy_symbol_paths.contains(&path) {
                    // Only graph-only targets need a request-local override.
                    let owned = plan.make_mut::<COUNT>(clones);
                    owned.symbols = request.plan.symbols;
                    owned.legacy_symbol_paths.insert(path);
                }
                plan
            });
        }
        if let Some(playwright) = &request.playwright {
            for path in playwright.paths() {
                plans
                    .entry(normalize(path))
                    .or_insert_with(|| DemandPlan::borrowed(graph_plan));
            }
            for path in playwright.source_files().iter() {
                let plan = plans
                    .entry(normalize(path))
                    .or_insert_with(|| DemandPlan::borrowed(graph_plan));
                // Staged Playwright decides whether source imports are needed
                // after test facts exist. Collect that request-local superset
                // in the same parse; map assembly retains the exact graph plan.
                if !plan.as_ref().graph.imports {
                    plan.make_mut::<COUNT>(clones).graph.imports = true;
                }
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
            let facts = plan.as_ref();
            if crate::codebase::dependencies::extract::is_indexable(&path)
                || facts.production_import_paths.contains(&path)
                || facts.storybook && is_mdx_file(&path)
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

pub(super) fn normalize(path: &std::path::Path) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(path)
}
