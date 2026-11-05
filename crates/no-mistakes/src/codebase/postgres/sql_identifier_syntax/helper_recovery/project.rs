//! Resolve only exact per-application sinks; ordinary syntax remains shared.
use super::super::Input;
use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    value::Value,
    values::{Cache, Engine},
    ApplicationKey, Recovered,
};
use crate::codebase::{
    check_facts::{
        sql_source_prerequisites::{source_demanded, source_failed, SqlProjection, SqlSourceKey},
        CheckFileFacts,
    },
    ts_source::FileIdMap,
};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};
mod finalize;

pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf>,
) {
    let (mut updates, mut failed_sources) = super::transports::project(files, &resolve);
    let mut caches = BTreeMap::<ApplicationKey, Cache>::new();
    for (path, file) in &*files {
        if file.parse_error.is_some() || file.ts.parse_error.is_some() {
            continue;
        }
        let models = &file.sql_identifier_syntax.recovery;
        for application in file.sql_identifier_syntax.application_keys() {
            let profile = &application.profile;
            let cache = caches.entry(application.clone()).or_default();
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
                profile: Some(profile),
                files,
                resolve: &tracked_resolve,
            };
            let Some(calls) = file.sql_identifier_syntax.calls_for(&application) else {
                continue;
            };
            let Some(sinks) = models.sinks.get(profile) else {
                continue;
            };
            for call in calls {
                let partial = application.rule == "postgres-no-test-state-mutation"
                    && finalize::finite_template_candidate(&call.input);
                if !(matches!(call.input, Input::Unknown) || partial) || !sinks.contains(&call.span)
                {
                    continue;
                }
                let Ok(index) = models
                    .calls
                    .binary_search_by_key(&call.span, |call| call.span)
                else {
                    continue;
                };
                let Some(argument) = models.calls[index].arguments.first() else {
                    continue;
                };
                let mut engine = Engine::new(&catalog, cache, Limits::default());
                let iteration = models.calls[index].caller.and_then(|caller| {
                    let rows = models.iterators.get(&caller)?;
                    let parameters = &models.functions.get(&caller)?.lowered.program.params;
                    let [parameter] = parameters.as_slice() else {
                        return None;
                    };
                    let id = parameter.name.strip_prefix("binding:")?.parse().ok()?;
                    (!models.writes.contains(&id)).then_some((&parameter.name, rows))
                });
                let values = match iteration {
                    Some((parameter, rows)) => {
                        engine.evaluate_iteration(path, argument, call.span, parameter, rows)
                    }
                    None => engine.evaluate_at(path, argument, Some(call.span)),
                };
                let mut recovered = if let Some(values) = values {
                    let mut recovered = Recovered {
                        residual: engine.residual(),
                        ..Default::default()
                    };
                    for value in values {
                        match value {
                            Value::Sql(text) | Value::Text(text) => {
                                recovered.residual |= !text.valid_binds();
                                recovered.alternatives.push(text.raw)
                            }
                            _ => recovered.residual = true,
                        }
                    }
                    recovered
                } else {
                    Recovered {
                        residual: true,
                        ..Default::default()
                    }
                };
                drop(engine);
                let mut call_failures = cache.take_observed_sources();
                if recovered.alternatives.is_empty() {
                    let Some(executors) = models.executors.get(profile) else {
                        finalize::push_failures(
                            &mut failed_sources,
                            path,
                            &application,
                            call.span,
                            call_failures,
                        );
                        continue;
                    };
                    let mut engine = Engine::new(&catalog, cache, Limits::default());
                    let recovered_site = engine.site(path, &models.calls[index], executors);
                    drop(engine);
                    let Some(recovered_site) = recovered_site else {
                        call_failures.extend(cache.take_observed_sources());
                        finalize::push_failures(
                            &mut failed_sources,
                            path,
                            &application,
                            call.span,
                            call_failures,
                        );
                        continue;
                    };
                    recovered = recovered_site;
                    call_failures.extend(cache.take_observed_sources());
                }
                finalize::push_failures(
                    &mut failed_sources,
                    path,
                    &application,
                    call.span,
                    call_failures,
                );
                if !recovered.alternatives.is_empty() && (!partial || recovered.complete()) {
                    updates.push((
                        path.clone(),
                        application.clone(),
                        call.span,
                        Arc::new(recovered),
                    ));
                }
            }
        }
    }
    finalize::apply(files, updates, failed_sources);
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

#[cfg(test)]
mod source_failure_tests;
