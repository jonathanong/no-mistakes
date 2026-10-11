use super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) struct BatchCheckFactRequest {
    pub(crate) root: PathBuf,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) graph_files: Vec<PathBuf>,
    pub(crate) plan: CheckFactPlan,
    pub(crate) playwright: Option<PlaywrightFactPlan>,
    pub(crate) sources: Arc<crate::codebase::ts_source::SourceStore>,
}

struct FactDemand {
    request: usize,
    plan: CheckFactPlan,
}

type CollectedDemand = (PathBuf, Vec<FactDemand>, Vec<Option<super::CheckFileFacts>>);

pub(crate) fn collect_check_fact_batch_with_session(
    session: &crate::codebase::analysis_session::AnalysisSession,
    requests: Vec<BatchCheckFactRequest>,
) -> Vec<CheckFactMap> {
    // Runner evaluation may visit helper programs before facts are collected.
    // Only already-requested runner plans are evaluated; ordinary source
    // queries do not activate runner discovery.
    let mut runner_plans = crate::fx::FxHashSet::default();
    for runner in requests
        .iter()
        .filter_map(|request| request.plan.integration_runner_configs.as_ref())
    {
        if runner_plans.insert(Arc::as_ptr(runner)) {
            let _ = runner.parse_all();
        }
    }
    let demands = demands_by_path(&requests);
    let cache = crate::ast::current_request_parse_cache();
    let mut collected = Vec::new();
    let demands = demands
        .into_iter()
        .filter_map(|(path, demands)| {
            if cache
                .as_ref()
                .is_some_and(|cache| cache.contains_path(&path))
            {
                // OXC programs cannot cross threads. Project every demanded variant
                // while the preparing thread still owns its config/helper AST.
                collected.extend(collect_demand(session, &requests, path, demands));
                None
            } else {
                Some((path, demands))
            }
        })
        .collect::<Vec<_>>();
    collected.extend(
        demands
            .into_par_iter()
            .map(|(path, demands)| collect_demand(session, &requests, path, demands))
            .while_some()
            .collect::<Vec<_>>(),
    );
    let mut precollected = (0..requests.len())
        .map(|index| {
            crate::codebase::ts_source::FileIdMap::with_inventory(Arc::clone(
                requests[index].sources.inventory(),
            ))
        })
        .collect::<Vec<_>>();
    for (path, demands, facts) in collected
        .into_iter()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
    {
        for (demand, facts) in demands.into_iter().zip(facts) {
            if let Some(facts) = facts {
                precollected[demand.request].insert(path.clone(), facts);
            }
        }
    }
    requests
        .into_iter()
        .zip(precollected)
        .map(|(request, facts)| {
            super::collect::collect_check_facts_with_precollected_file_facts(
                session,
                &request.root,
                (request.files, request.graph_files),
                request.plan,
                request.playwright,
                request.sources,
                facts,
            )
        })
        .collect()
}

fn collect_demand(
    session: &crate::codebase::analysis_session::AnalysisSession,
    requests: &[BatchCheckFactRequest],
    path: PathBuf,
    demands: Vec<FactDemand>,
) -> Option<CollectedDemand> {
    crate::invocation::check_timeout().ok().map(|()| {
        crate::ast::with_request_parse_cache(|| {
            let variants = demands
                .iter()
                .map(|demand| {
                    let request = &requests[demand.request];
                    super::file::CheckFactVariant {
                        root: &request.root,
                        plan: &demand.plan,
                        playwright: request.playwright.as_ref(),
                    }
                })
                .collect::<Vec<_>>();
            let facts =
                super::file::collect_file_fact_variants_with_session(session, &path, &variants);
            (path, demands, facts)
        })
    })
}

fn demands_by_path(requests: &[BatchCheckFactRequest]) -> BTreeMap<PathBuf, Vec<FactDemand>> {
    let mut demands = BTreeMap::<PathBuf, Vec<FactDemand>>::new();
    for (request_id, request) in requests
        .iter()
        .enumerate()
        .take_while(|_| crate::invocation::check_timeout().is_ok())
    {
        let graph_only = super::graph_only_files(&request.files, &request.graph_files);
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
        for path in graph_only {
            plans.entry(normalize(&path)).or_insert(graph_plan.clone());
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
                || plan.storybook && super::is_mdx_file(&path)
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
