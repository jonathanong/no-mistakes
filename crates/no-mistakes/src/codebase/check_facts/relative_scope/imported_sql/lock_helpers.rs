use super::program::Shared;
use crate::codebase::postgres::embedded::walk::lock_helpers::{Invocation, Parameter};
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlOptions};
use std::path::{Path, PathBuf};

pub(super) fn complete(
    shared: &mut Shared<'_, impl FnMut(&str, &Path) -> Option<PathBuf>>,
    path: &Path,
    options: &EmbeddedSqlOptions,
    invocation: &Invocation,
) -> Vec<EmbeddedSqlCall> {
    if !invocation.resolve_helper {
        return vec![invocation.call.clone()];
    }
    let summary = shared
        .find(path, &invocation.name, None)
        .and_then(|(path, name)| {
            shared
                .files
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
            call.lock_helper = Some(Box::new((invocation.name.clone(), query.line)));
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
    calls
}
