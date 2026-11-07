use super::{
    locate::{Located, Locator},
    CheckFileFacts,
};
use crate::codebase::postgres::embedded::walk::lock_helpers::Parameter;
use crate::codebase::postgres::EmbeddedSqlKind;
use crate::codebase::ts_source::FileIdMap;
use std::path::{Path, PathBuf};

pub(super) fn project(
    files: &mut FileIdMap<CheckFileFacts>,
    resolve: impl FnMut(&str, &Path) -> Option<PathBuf>,
) {
    if !files.into_iter().any(|(_, file)| {
        file.embedded_sql
            .iter()
            .any(|(_, embedded)| !embedded.imported_lock_helpers.is_empty())
    }) {
        return;
    }
    let mut locator = Locator::new(files, resolve);
    let mut completed = Vec::new();
    for (path, file) in &*files {
        let imports: crate::fx::FxHashMap<_, _> = file
            .ts
            .imported_bindings
            .iter()
            .map(|binding| (binding.local.as_str(), binding))
            .collect();
        for (profile, (options, embedded)) in file.embedded_sql.iter().enumerate() {
            for invocation in &embedded.imported_lock_helpers {
                let summary = imports
                    .get(invocation.name.as_str())
                    .and_then(|binding| {
                        locator
                            .target(&binding.specifier, path)
                            .map(|target| (target, binding.imported.as_str()))
                    })
                    .and_then(|(target, name)| match locator.locate(&target, name) {
                        Located::Found(path, name) => Some((path, name)),
                        _ => None,
                    })
                    .and_then(|(path, name)| {
                        files
                            .get(&path)
                            .and_then(|file| {
                                file.embedded_sql
                                    .iter()
                                    .find(|(profile, _)| profile == options)
                            })
                            .and_then(|(_, embedded)| embedded.lock_helper_summaries.get(&name))
                    });
                let mut calls = Vec::new();
                if let Some(summary) = summary {
                    for query in summary
                        .calls
                        .iter()
                        .filter(|query| query.executor == invocation.executor)
                    {
                        let mut call = invocation.call.clone();
                        call.sql_text = Some(query.sql.clone());
                        call.kind = EmbeddedSqlKind::Inline;
                        call.lock_helper = Some((invocation.name.clone(), query.line));
                        call.bound_parameters = query
                            .parameters
                            .iter()
                            .map(|parameter| match parameter {
                                Parameter::Argument(index) => {
                                    invocation.arguments.get(*index).cloned().flatten()
                                }
                                Parameter::Literal(value) => Some(value.clone()),
                                Parameter::Unknown => None,
                            })
                            .collect();
                        calls.push(call);
                    }
                }
                if calls.is_empty() {
                    calls.push(invocation.call.clone());
                }
                completed.push((path.clone(), profile, invocation.span, calls));
            }
        }
    }
    drop(locator);
    for (path, profile, span, calls) in completed {
        let embedded = &mut files.get_mut(&path).unwrap().embedded_sql[profile].1;
        for call in calls {
            embedded.calls.push(call);
            embedded.call_spans.push(span);
        }
        embedded.imported_lock_helpers.clear();
    }
}
