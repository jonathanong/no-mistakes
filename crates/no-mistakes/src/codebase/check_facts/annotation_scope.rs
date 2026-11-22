use super::batch::{collect_demand, BatchCheckFactRequest, CollectedDemand, FactDemand};
use super::{CheckFactPlan, CheckFileFacts, PlaywrightFactPlan};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::extract::is_indexable;
use crate::codebase::ts_source::FileIdMap;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) fn eligible(plan: &CheckFactPlan, playwright: Option<&PlaywrightFactPlan>) -> bool {
    plan.query_annotation
        && plan.query_annotation_catalog.is_some()
        && plan
            .query_annotation_options
            .as_ref()
            .is_some_and(|options| {
                !options.is_empty()
                    && options.iter().all(|profile| {
                        let Some(scans) = plan.annotation_scans().get(profile) else {
                            return false;
                        };
                        let mut seeds = scans.iter().filter(|scan| scan.annotation_seed);
                        seeds.clone().next().is_some()
                            && seeds.all(|scan| !scan.annotation_unrestricted())
                    })
            })
        && plan.legacy_symbol_paths.is_empty()
        && plan.integration_runner_configs.is_none()
        && playwright.is_none()
}

pub(super) fn collect(
    session: &AnalysisSession,
    requests: &[BatchCheckFactRequest],
    pending: &mut BTreeMap<PathBuf, Vec<FactDemand>>,
) -> Option<(usize, FileIdMap<CheckFileFacts>, u32, Vec<CollectedDemand>)> {
    let mut scoped = requests
        .iter()
        .enumerate()
        .filter(|(_, request)| request.plan.query_annotation);
    let (id, request) = scoped.next()?;
    if scoped.next().is_some()
        || !eligible(&request.plan, request.playwright.as_ref())
        || requests.iter().any(|request| {
            request.playwright.is_some() || request.plan.integration_runner_configs.is_some()
        })
        || !pending.iter().any(|(path, variants)| {
            variants
                .iter()
                .any(|variant| variant.request == id && variant.plan.query_annotation)
                && !is_scope_seed(&request.root, path, &request.plan)
        })
    {
        return None;
    }
    let paths = pending
        .iter()
        .filter(|(_, variants)| variants.iter().any(|variant| variant.request == id))
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    let parent_cache = crate::ast::current_request_parse_cache();
    let workers = rayon::current_num_threads().min(paths.len());
    let observer = crate::diagnostics::current();
    let collected = rayon::ThreadPoolBuilder::new()
        .num_threads(workers)
        .build_scoped(
            |thread| crate::ast::with_owned_request_parse_cache(|| thread.run()),
            |pool| {
                let mut cached = Vec::new();
                let mut tasks = Vec::new();
                for path in paths {
                    let mut variants = pending.remove(&path).unwrap();
                    for variant in &mut variants {
                        if variant.request == id {
                            variant.plan.defer_query_annotation = true;
                        }
                    }
                    let task = (path, variants);
                    if parent_cache
                        .as_ref()
                        .is_some_and(|cache| cache.contains_path(&task.0))
                    {
                        cached.push(task);
                    } else {
                        tasks.push(std::sync::Mutex::new(Some(task)));
                    }
                }
                let next = AtomicUsize::new(0);
                let (mut owned, mut cached) =
                    crate::perf_trace::trace("postgres.annotation.prepare.extract", || {
                        let owned = pool.broadcast(|_| {
                            crate::diagnostics::with_observer(observer.clone(), || {
                                std::iter::from_fn(|| {
                                    tasks.get(next.fetch_add(1, Ordering::Relaxed))
                                })
                                .filter_map(|task| {
                                    let (path, variants) = task.lock().unwrap().take().unwrap();
                                    collect_demand(session, requests, path, variants)
                                })
                                .collect::<Vec<_>>()
                            })
                        });
                        let cached = cached
                            .into_iter()
                            .filter_map(|(path, variants)| {
                                collect_demand(session, requests, path, variants)
                            })
                            .collect::<Vec<_>>();
                        (owned, cached)
                    });
                let mut files =
                    FileIdMap::with_inventory(std::sync::Arc::clone(request.sources.inventory()));
                for (path, variants, facts) in owned.iter_mut().flatten().chain(cached.iter_mut()) {
                    for (variant, facts) in variants.iter().zip(facts) {
                        if variant.request == id {
                            if let Some(facts) = facts.take() {
                                files.insert(path.clone(), facts);
                            }
                        }
                    }
                }
                let mut failures = crate::fx::fx_map();
                let projected = super::relative_scope::project_with_annotation_overlay(
                    session,
                    &request.root,
                    &request.sources,
                    request.plan.query_annotation_catalog.as_deref(),
                    &mut files,
                    Some(&mut |files, demand| {
                        let mut results = pool.broadcast(|ctx| {
                            crate::diagnostics::with_observer(observer.clone(), || {
                                let results =
                                    overlay::lower(session, files, &demand, &owned[ctx.index()]);
                                crate::ast::clear_request_parse_cache();
                                results
                            })
                        });
                        results.push(overlay::lower(session, files, &demand, &cached));
                        for (path, _, _) in &cached {
                            crate::ast::evict_request_parse_cache_path(path);
                        }
                        failures = overlay::install(request, files, results.into_iter().flatten());
                        failures.is_empty()
                    }),
                );
                for (path, _, _) in &cached {
                    crate::ast::evict_request_parse_cache_path(path);
                }
                (
                    id,
                    files,
                    projected,
                    owned
                        .into_iter()
                        .flatten()
                        .chain(cached)
                        .map(|task| {
                            if let Some(error) = failures.get(&task.0) {
                                return overlay::fail_other_variants(requests, task, error);
                            }
                            task
                        })
                        .collect(),
                )
            },
        )
        .ok();
    collected
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
pub(super) fn has_excluded_file(root: &Path, files: &[PathBuf], plan: &CheckFactPlan) -> bool {
    files.iter().any(|path| !is_scope_seed(root, path, plan))
}

fn is_scope_seed(root: &Path, path: &Path, plan: &CheckFactPlan) -> bool {
    if !is_indexable(path) {
        return true;
    }
    let relative = crate::codebase::ts_source::relative_slash_path(root, path);
    plan.annotation_scans()
        .values()
        .flatten()
        .any(|scan| scan.includes_annotation(path, &relative))
}

mod overlay;
