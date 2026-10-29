//! Resolve only exact per-application sinks; ordinary syntax remains shared.
use super::super::Input;
use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    value::Value,
    values::{Cache, Engine},
    ApplicationKey, Recovered,
};
use crate::codebase::{check_facts::CheckFileFacts, ts_source::FileIdMap};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

pub(crate) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl Fn(&str, &Path) -> Option<PathBuf>,
) {
    let mut updates = super::transports::project(files, &resolve);
    let mut caches = BTreeMap::<ApplicationKey, Cache>::new();
    for (path, file) in &*files {
        if file.parse_error.is_some() || file.ts.parse_error.is_some() {
            continue;
        }
        let models = &file.sql_identifier_syntax.recovery;
        for application in file.sql_identifier_syntax.application_keys() {
            let profile = &application.profile;
            let catalog = PreparedCatalog {
                application: Some(&application),
                profile: Some(profile),
                files,
                resolve: &resolve,
            };
            let Some(calls) = file.sql_identifier_syntax.calls_for(&application) else {
                continue;
            };
            let Some(sinks) = models.sinks.get(profile) else {
                continue;
            };
            let cache = caches.entry(application.clone()).or_default();
            for call in calls {
                let partial = application.rule == "postgres-no-test-state-mutation"
                    && finite_template_candidate(&call.input);
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
                if recovered.alternatives.is_empty() {
                    let Some(executors) = models.executors.get(profile) else {
                        continue;
                    };
                    let mut engine = Engine::new(&catalog, cache, Limits::default());
                    let Some(recovered_site) = engine.site(path, &models.calls[index], executors)
                    else {
                        continue;
                    };
                    recovered = recovered_site;
                }
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
    for (path, application, span, recovered) in updates {
        let Some(file) = files.get_mut(&path) else {
            continue;
        };
        if !file
            .sql_identifier_syntax
            .application_calls
            .contains_key(&application)
        {
            let Some(calls) = file
                .sql_identifier_syntax
                .calls_for(&application)
                .map(<[_]>::to_vec)
            else {
                continue;
            };
            file.sql_identifier_syntax
                .application_calls
                .insert(application.clone(), calls);
        }
        let calls = file
            .sql_identifier_syntax
            .application_calls
            .get_mut(&application)
            .unwrap();
        if let Ok(index) = calls.binary_search_by_key(&span, |call| call.span) {
            calls[index].input = Input::Recovered(recovered);
        }
    }
}

fn finite_template_candidate(input: &Input) -> bool {
    let Input::Sql {
        text,
        holes,
        parameterized: None,
        ..
    } = input
    else {
        return false;
    };
    let first = holes
        .iter()
        .filter_map(|(marker, _)| text.find(marker))
        .min();
    // Preserve the existing complete-statement prefix path. In particular,
    // an invalid recovered suffix must never erase a known prefix finding.
    first.is_some_and(|first| !text[..first].contains(';'))
}
