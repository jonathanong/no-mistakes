use super::evaluate::{Evaluator, File, Value};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::sql_requires_query_annotation;
use crate::codebase::ts_source::FileIdMap;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Resolve owned syntax against the request's canonical module facts and
/// resolver. No source reads or parsing occur in this projection.
pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf> + Sync,
) {
    let originals = files
        .into_iter()
        .map(|(path, _)| {
            (
                crate::codebase::ts_resolver::normalize_path(path),
                path.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
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
                    crate::codebase::ts_resolver::normalize_path(path),
                    File {
                        facts,
                        ts: &file.ts,
                        executors: embedded.call_starts.clone(),
                    },
                ))
            })
            .collect::<BTreeMap<_, _>>();
        let mut per_file = collected
            .par_iter()
            .map(|(path, _)| {
                let mut evaluator = Evaluator {
                    files: &collected,
                    resolve: &resolve,
                    events: BTreeMap::new(),
                    scopes: Vec::new(),
                };
                evaluator.run(path);
                (path.clone(), evaluator.events)
            })
            .collect::<Vec<_>>();
        per_file.sort_by(|left, right| left.0.cmp(&right.0));
        let mut events = BTreeMap::<_, Vec<_>>::new();
        for (_, file_events) in per_file {
            for (call, values) in file_events {
                events.entry(call).or_default().extend(values);
            }
        }
        for ((path, start), events) in events {
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
            facts.calls.insert(start, value);
        }
    }
}

fn prefix(value: Value) -> Option<String> {
    let Value::Prefix(text, complete) = value else {
        return None;
    };
    let start = text.trim_start();
    if complete
        || super::super::annotation::has_leading_query_annotation(&text)
        || (!start.is_empty() && !start.starts_with('/') && sql_requires_query_annotation(&text))
        || (start.starts_with('/') && start.len() > 1 && !start.starts_with("/*"))
        || (start.starts_with("/*") && start.contains("*/") && sql_requires_query_annotation(&text))
    {
        Some(text)
    } else {
        None
    }
}
