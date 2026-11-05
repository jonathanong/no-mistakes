//! A raw executor escape clears only after the exact enclosing helper is checked.
use super::super::{Input, Span};
use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    values::{Cache, Engine},
    ApplicationKey, Recovered,
};
use crate::codebase::{
    check_facts::{
        sql_source_prerequisites::{
            source_demanded, source_failed, FailedSqlSources, SqlProjection, SqlSourceKey,
        },
        CheckFileFacts,
    },
    ts_source::FileIdMap,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

type Update = (PathBuf, ApplicationKey, Span, Arc<Recovered>);
type Failure = (PathBuf, ApplicationKey, Span, FailedSqlSources);
pub(super) fn project(
    files: &FileIdMap<CheckFileFacts>,
    resolve: &dyn Fn(&str, &Path) -> Option<PathBuf>,
) -> (Vec<Update>, Vec<Failure>) {
    let mut updates = Vec::new();
    let mut failures = Vec::new();
    for (path, file) in files {
        if file.parse_error.is_some() || file.ts.parse_error.is_some() {
            continue;
        }
        let models = &file.sql_identifier_syntax.recovery;
        for application in file.sql_identifier_syntax.application_keys() {
            let profile = &application.profile;
            let Some(calls) = file.sql_identifier_syntax.calls_for(&application) else {
                continue;
            };
            let Some(executors) = models.executors.get(profile) else {
                continue;
            };
            let mut cache = Cache::default();
            let observed = cache.observed_sources.clone();
            let tracked_resolve = |specifier: &str, from: &Path| {
                let target = resolve(specifier, from);
                if let Some(path) = target
                    .as_deref()
                    .filter(|path| failed_source_is_demanded(files, path, &application))
                {
                    observed.borrow_mut().push(path.to_path_buf());
                }
                target
            };
            let catalog = PreparedCatalog {
                application: Some(&application),
                files,
                profile: Some(profile),
                resolve: &tracked_resolve,
            };
            let mut completed = BTreeMap::<Span, (Option<Arc<Recovered>>, FailedSqlSources)>::new();
            for call in calls {
                if !matches!(call.input, Input::Unknown)
                    || models
                        .sinks
                        .get(profile)
                        .is_some_and(|sinks| sinks.contains(&call.span))
                {
                    continue;
                }
                // The nearest argument owner matters: an outer modeled helper
                // cannot bless an executor first handed to an opaque inner call.
                let owner = models
                    .argument_owners
                    .get(&call.span)
                    .and_then(|index| models.calls.get(*index))
                    .filter(|model| {
                        model.arguments.iter().any(|argument| {
                            argument
                                .references
                                .iter()
                                .any(|(id, at)| executors.contains(id) && *at == call.span)
                        })
                    });
                let Some(owner) = owner else {
                    continue;
                };
                let mut call_failures = FailedSqlSources::new();
                if callbacks::blocks(&catalog, path, &application, models, owner) {
                    call_failures.extend(cache.take_observed_sources());
                    push_failures(&mut failures, path, &application, call.span, call_failures);
                    continue;
                }
                if typed::accepts(
                    &catalog,
                    path,
                    &application,
                    models,
                    owner,
                    call.span,
                    executors,
                ) {
                    updates.push((
                        path.clone(),
                        application.clone(),
                        call.span,
                        Arc::new(Recovered {
                            nonexecuting: true,
                            ..Default::default()
                        }),
                    ));
                    call_failures.extend(cache.take_observed_sources());
                    push_failures(&mut failures, path, &application, call.span, call_failures);
                    continue;
                }
                let (recovered, cached_failures) = if let Some(cached) = completed.get(&owner.span)
                {
                    (cached.0.clone(), cached.1.clone())
                } else {
                    let mut engine = Engine::new(&catalog, &mut cache, Limits::default());
                    let recovered = engine.transport(path, owner, executors).map(Arc::new);
                    drop(engine);
                    let sources = cache.take_observed_sources();
                    completed.insert(owner.span, (recovered.clone(), sources.clone()));
                    (recovered, sources)
                };
                call_failures.extend(cached_failures);
                push_failures(&mut failures, path, &application, call.span, call_failures);
                let Some(recovered) = recovered else {
                    continue;
                };
                if recovered.complete() || !recovered.alternatives.is_empty() {
                    updates.push((path.clone(), application.clone(), call.span, recovered));
                }
            }
        }
    }
    (updates, failures)
}

fn push_failures(
    out: &mut Vec<Failure>,
    owner: &Path,
    application: &ApplicationKey,
    span: Span,
    failures: FailedSqlSources,
) {
    if !failures.is_empty() {
        out.push((owner.to_path_buf(), application.clone(), span, failures));
    }
}

fn failed_source_is_demanded(
    files: &FileIdMap<CheckFileFacts>,
    path: &Path,
    application: &ApplicationKey,
) -> bool {
    if !source_failed(files, path) {
        return false;
    }
    let key = SqlSourceKey::application(
        SqlProjection::LowSyntax,
        &application.profile,
        &application.rule,
        application.index,
    );
    source_demanded(files, path, &key)
}

mod typed;

mod callbacks;

#[cfg(test)]
mod source_failure_tests;
