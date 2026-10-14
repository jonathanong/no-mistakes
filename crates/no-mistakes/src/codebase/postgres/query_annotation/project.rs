use super::evaluate::{File, Value};
use super::modules::Module;
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::sql_requires_query_annotation;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod evaluator;
mod output;
use output::{prefix, projected_sql};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectedSql {
    pub text: String,
    pub complete: bool,
    pub only_value_placeholders: bool,
}

pub(crate) type ProjectedCalls =
    BTreeMap<(PathBuf, (u32, u32)), Result<Vec<ProjectedSql>, &'static str>>;

/// Resolve owned syntax against the request's canonical module facts and
/// resolver. No source reads or parsing occur in this projection.
pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
) {
    let _ = project_impl(files, resolve, false);
}

pub(crate) fn project_executed(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
) -> ProjectedCalls {
    project_impl(files, resolve, true)
}

fn project_impl(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
    collect_executed: bool,
) -> ProjectedCalls {
    let mut projected = ProjectedCalls::new();
    let originals = files
        .into_iter()
        .map(|(path, _)| (normalize_path(path), path.clone()))
        .collect::<FxHashMap<_, _>>();
    let mut profiles = files
        .into_iter()
        .flat_map(|(_, file)| {
            file.query_annotation
                .iter()
                .map(|(options, _)| options.clone())
        })
        .collect::<Vec<_>>();
    profiles.sort();
    profiles.dedup();
    let modules = files
        .into_iter()
        .filter(|(_, file)| file.parse_error.is_none() && !file.query_annotation.is_empty())
        .map(|(path, file)| (path.clone(), Arc::new(Module::new(Arc::clone(&file.ts)))))
        .collect::<FxHashMap<_, _>>();
    for options in profiles {
        let collected = files
            .into_iter()
            .filter_map(|(path, file)| {
                if file.parse_error.is_some() {
                    return None;
                }
                let (_, facts) = file
                    .query_annotation
                    .iter()
                    .find(|(profile, _)| profile == &options)?;
                let (_, embedded) = file
                    .embedded_sql
                    .iter()
                    .find(|(profile, _)| profile == &options)
                    .expect("annotation summaries borrow their embedded profile");
                Some((
                    normalize_path(path),
                    File {
                        module: Arc::clone(&modules[path]),
                        facts,
                        executors: embedded.call_spans.iter().copied().collect(),
                    },
                ))
            })
            .collect::<FxHashMap<_, _>>();
        let untrusted_array_intrinsics = collected
            .values()
            .any(|file| file.facts.array_prototype_exposed);
        let observer = crate::diagnostics::current();
        let mut per_file = collected
            .par_iter()
            .map(|(path, _)| {
                let events = crate::diagnostics::with_observer(observer.clone(), || {
                    let mut evaluator =
                        evaluator::new(&collected, &resolve, untrusted_array_intrinsics);
                    evaluator.run(path);
                    evaluator.events
                });
                (path.clone(), events)
            })
            .collect::<Vec<_>>();
        per_file.sort_by(|left, right| left.0.cmp(&right.0));
        let mut events = BTreeMap::<_, Vec<_>>::new();
        for (_, file_events) in per_file {
            for (call, values) in file_events {
                events.entry(call).or_default().extend(values);
            }
        }
        for ((path, span), events) in events {
            // An exported helper can be called both locally with known SQL and
            // externally with an unknown argument. The annotation rule keeps
            // its contextual preference; executed SQL must include every
            // reachable invocation before it certifies a complete statement.
            let contextual = !collect_executed && events.iter().any(|(generic, _)| !generic);
            let values = events
                .into_iter()
                .filter(|(generic, _)| !contextual || !generic)
                .map(|(_, value)| value)
                .collect::<Vec<_>>();
            if values
                .iter()
                .all(|value| matches!(value, Value::Unsupported))
            {
                // Legacy lexical SQL facts remain authoritative where this
                // additive helper projection does not model the scope.
                continue;
            }
            if collect_executed {
                let mut alternatives = Vec::new();
                let mut reason = None;
                for value in &values {
                    if let Some(candidates) = projected_sql(value) {
                        for candidate in candidates {
                            if !candidate.complete && !candidate.only_value_placeholders {
                                reason = Some("unresolved-sql-fragment");
                                break;
                            }
                            if !alternatives.contains(&candidate) {
                                alternatives.push(candidate);
                                if alternatives.len() > super::super::MAX_EMBEDDED_SQL_VARIANTS {
                                    reason = Some("variant-limit");
                                    break;
                                }
                            }
                        }
                    } else {
                        reason = Some(output::incomplete_reason(value));
                        break;
                    }
                }
                projected.insert((path.clone(), span), reason.map_or(Ok(alternatives), Err));
            }
            let values = values.into_iter().map(prefix).collect::<Vec<_>>();
            // Every analyzable invocation must satisfy the rule. One favorable
            // callback callsite must never hide another unannotated/opaque one.
            let value = values
                .iter()
                .flatten()
                .find(|value| sql_requires_query_annotation(value))
                .cloned()
                .or_else(|| {
                    if values.iter().any(Option::is_none) {
                        None
                    } else {
                        values.into_iter().flatten().next()
                    }
                });
            let file = files
                .get_mut(&originals[&path])
                .expect("projected source remains in the prepared map");
            let (_, facts) = file
                .query_annotation
                .iter_mut()
                .find(|(profile, _)| profile == &options)
                .expect("projected annotation profile remains prepared");
            facts.calls.insert(span, value);
        }
    }
    projected
}

#[cfg(test)]
mod tests;
