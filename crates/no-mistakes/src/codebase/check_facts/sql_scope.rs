use super::batch::{collect_demand, BatchCheckFactRequest, CollectedDemand, FactDemand};
use super::{CheckFactPlan, PlaywrightFactPlan};
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::ts_resolver::ScopedImportResolver;
use crate::fx::PathSet;
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub(super) fn eligible(plan: &CheckFactPlan, playwright: Option<&PlaywrightFactPlan>) -> bool {
    #[cfg(test)]
    if tests::BROAD.with(std::cell::Cell::get) {
        return false;
    }
    plan.embedded_sql
        && !plan.embedded_sql_options.is_empty()
        && plan.query_annotation_catalog.is_some()
        && !plan.query_annotation
        && !plan
            .embedded_sql_rule_ids
            .values()
            .flatten()
            .any(|scan| scan.unrestricted())
        && !plan.query_annotation_heads
        && plan.sql_identifier_profiles.is_empty()
        && plan.integration_runner_configs.is_none()
        && playwright.is_none()
        && plan.embedded_sql_options.iter().all(|profile| {
            profile.sql_literal_helpers.is_empty()
                && !profile.track_transaction_helpers
                && plan
                    .embedded_sql_rule_ids
                    .get(profile)
                    .is_some_and(|scans| !scans.is_empty())
        })
        && !crate::ast::current_request_parse_cache().is_some_and(|cache| cache.entry_count() > 0)
}

pub(super) fn collect(
    session: &AnalysisSession,
    requests: &[BatchCheckFactRequest],
    demands: &mut BTreeMap<PathBuf, Vec<FactDemand>>,
) -> Vec<CollectedDemand> {
    let mut sql = requests.iter().enumerate().filter(|(_, request)| {
        request.plan.embedded_sql
            || !request.plan.embedded_sql_options.is_empty()
            || request.plan.query_annotation
            || request.plan.query_annotation_heads
            || !request.plan.sql_identifier_profiles.is_empty()
    });
    let Some((id, request)) = sql.next() else {
        return Vec::new();
    };
    if sql.next().is_some()
        || !eligible(&request.plan, request.playwright.as_ref())
        || !session
            .existing_sources_for(&request.root)
            .is_some_and(|sources| std::sync::Arc::ptr_eq(&sources, &request.sources))
        || requests.iter().any(|request| {
            request.playwright.is_some() || request.plan.integration_runner_configs.is_some()
        })
    {
        return Vec::new();
    }
    let seeds = demands
        .iter()
        .filter(|(path, variants)| {
            variants
                .iter()
                .any(|variant| variant.request == id && variant.plan.embedded_sql)
                && request
                    .plan
                    .embedded_sql_rule_ids
                    .values()
                    .flatten()
                    .any(|scan| {
                        scan.paths
                            .includes(&crate::codebase::ts_source::relative_slash_path(
                                &request.root,
                                path,
                            ))
                    })
        })
        .map(|(path, _)| path.clone())
        .collect::<BTreeSet<_>>();
    let total = demands
        .values()
        .filter(|variants| {
            variants
                .iter()
                .any(|variant| variant.request == id && variant.plan.embedded_sql)
        })
        .count();
    if seeds.len() == total {
        return Vec::new();
    }
    let visible = request
        .sources
        .inventory()
        .paths()
        .iter()
        .cloned()
        .collect::<PathSet>();
    let workspace = session.workspace(&request.root);
    let resolver = ScopedImportResolver::new_in_session(
        request.plan.query_annotation_catalog.as_ref().unwrap(),
        &visible,
        session,
    );
    let mut pending = seeds;
    let mut collected = Vec::new();
    let observer = crate::diagnostics::current();
    while !pending.is_empty() && crate::invocation::check_timeout().is_ok() {
        let wave = std::mem::take(&mut pending)
            .into_iter()
            .filter_map(|path| demands.remove(&path).map(|variants| (path, variants)))
            .collect::<Vec<_>>();
        let results = wave
            .into_par_iter()
            .filter_map(|(path, variants)| {
                crate::diagnostics::with_observer(observer.clone(), || {
                    collect_demand(session, requests, path, variants)
                })
            })
            .collect::<Vec<_>>();
        for result in &results {
            let Some(facts) = result.1.iter().zip(&result.2).find_map(|(variant, facts)| {
                (variant.request == id).then_some(facts.as_ref()).flatten()
            }) else {
                continue;
            };
            // These are the exact binding tables consumed by imported SQL projection.
            let specifiers = facts
                .ts
                .imported_bindings
                .iter()
                .filter(|binding| !binding.is_type_only)
                .map(|binding| binding.specifier.as_str())
                .chain(
                    facts
                        .ts
                        .exported_bindings
                        .iter()
                        .filter_map(|binding| binding.specifier.as_deref()),
                )
                .chain(facts.ts.star_reexport_specifiers.iter().map(String::as_str));
            for specifier in specifiers {
                let target = resolver.resolve(specifier, &result.0).or_else(|| {
                    workspace.resolve_specifier_from_file_visible(specifier, &result.0, &visible)
                });
                if let Some(target) = target.filter(|target| demands.contains_key(target)) {
                    pending.insert(target);
                }
            }
        }
        collected.extend(results);
    }
    for variants in demands.values_mut() {
        for variant in variants.iter_mut().filter(|variant| variant.request == id) {
            let original_ts = variant.plan.collected_ts_plan();
            variant.plan.graph.include(original_ts);
            variant.plan.embedded_sql = false;
            variant.plan.embedded_sql_options.clear();
            variant.plan.embedded_sql_head_calls_only.clear();
        }
    }
    collected
}

#[cfg(test)]
mod tests;
