//! A raw executor escape clears only after the exact enclosing helper is checked.
use super::super::{Input, Span};
use super::{
    budget::Limits,
    catalog::PreparedCatalog,
    values::{Cache, Engine},
    ApplicationKey, Recovered,
};
use crate::codebase::{check_facts::CheckFileFacts, ts_source::FileIdMap};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};

type Update = (PathBuf, ApplicationKey, Span, Arc<Recovered>);
pub(super) fn project(
    files: &FileIdMap<CheckFileFacts>,
    resolve: &dyn Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<Update> {
    let mut updates = Vec::new();
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
            let catalog = PreparedCatalog {
                application: Some(&application),
                files,
                profile: Some(profile),
                resolve,
            };
            let mut cache = Cache::default();
            let mut completed = BTreeMap::new();
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
                let recovered = completed.entry(owner.span).or_insert_with(|| {
                    Engine::new(&catalog, &mut cache, Limits::default())
                        .transport(path, owner, executors)
                        .map(Arc::new)
                });
                let Some(recovered) = recovered else {
                    continue;
                };
                if recovered.complete() || !recovered.alternatives.is_empty() {
                    updates.push((
                        path.clone(),
                        application.clone(),
                        call.span,
                        Arc::clone(recovered),
                    ));
                }
            }
        }
    }
    updates
}
