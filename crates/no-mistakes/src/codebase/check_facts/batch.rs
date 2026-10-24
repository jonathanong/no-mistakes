use super::{CheckFactMap, CheckFactPlan, PlaywrightFactPlan};
use rayon::prelude::*;
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

struct FactDemand<'a> {
    request: usize,
    plan: plan::DemandPlan<'a>,
}

type CollectedDemand<'a> = (
    PathBuf,
    Vec<FactDemand<'a>>,
    Vec<Option<super::CheckFileFacts>>,
);

#[cfg(feature = "test-instrumentation")]
pub(crate) mod benchmark;
mod demands;
mod plan;
#[cfg(feature = "test-instrumentation")]
mod test_support;
#[cfg(test)]
mod tests;

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
    let mut graph_plans = demands::graph_plans(&requests);
    let demands = demands::demands_by_path::<false>(&requests, &mut graph_plans, &mut 0);
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
    let observer = crate::diagnostics::current();
    collected.extend(
        demands
            .into_par_iter()
            .map(|(path, demands)| {
                crate::diagnostics::with_observer(observer.clone(), || {
                    collect_demand(session, &requests, path, demands)
                })
            })
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
    // Release graph-only Arc owners before assembling maps from the original plans.
    drop(graph_plans);
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

fn collect_demand<'a>(
    session: &crate::codebase::analysis_session::AnalysisSession,
    requests: &[BatchCheckFactRequest],
    path: PathBuf,
    demands: Vec<FactDemand<'a>>,
) -> Option<CollectedDemand<'a>> {
    crate::invocation::check_timeout().ok().map(|()| {
        crate::ast::with_request_parse_cache(|| {
            let variants = demands
                .iter()
                .map(|demand| {
                    let request = &requests[demand.request];
                    super::file::CheckFactVariant {
                        root: &request.root,
                        plan: demand.plan.as_ref(),
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
