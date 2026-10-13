use super::evaluate::{Evaluator, File, Value};
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

mod output;
use output::{prefix, projected_sql};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProjectedSql {
    pub text: String,
    pub complete: bool,
}

pub(crate) type ProjectedCalls = BTreeMap<(PathBuf, (u32, u32)), Option<Vec<ProjectedSql>>>;

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
        let observer = crate::diagnostics::current();
        let mut per_file = collected
            .par_iter()
            .map(|(path, _)| {
                let events = crate::diagnostics::with_observer(observer.clone(), || {
                    let mut evaluator = Evaluator {
                        files: &collected,
                        resolve: &resolve,
                        events: BTreeMap::new(),
                        scopes: Vec::new(),
                        modules: crate::fx::fx_map(),
                        active_module_initials: Default::default(),
                        active_callback_functions: Default::default(),
                        active_callback_executions: Default::default(),
                        next_builder: 0,
                        invalidated_builders: Default::default(),
                        builder_updates: Default::default(),
                        captured_bindings: Default::default(),
                        captured_binding_readers: Default::default(),
                        mapped_argument_owners: Default::default(),
                        mapped_parameter_indices: Default::default(),
                        deleted_argument_slots: Default::default(),
                        mapped_arguments: Default::default(),
                        fresh_mapped_parameters: Default::default(),
                        fresh_mapped_argument_bindings: Default::default(),
                        argument_objects: Default::default(),
                        argument_extra_slots: Default::default(),
                        definite_deleted_argument_slots: Default::default(),
                        recreated_argument_slots: Default::default(),
                        disconnected_argument_slots: Default::default(),
                    };
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
            let contextual = events.iter().any(|(generic, _)| !generic);
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
                let mut recoverable = true;
                for value in &values {
                    if let Some(candidates) = projected_sql(value) {
                        for candidate in candidates {
                            if !alternatives.contains(&candidate) {
                                alternatives.push(candidate);
                            }
                        }
                    } else {
                        recoverable = false;
                        break;
                    }
                }
                projected.insert((path.clone(), span), recoverable.then_some(alternatives));
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
