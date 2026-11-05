use super::super::{super::Input, ApplicationKey, Recovered};
use crate::codebase::{
    check_facts::{
        sql_source_prerequisites::{FailedSqlSources, SqlProjection, SqlSourceKey},
        CheckFileFacts,
    },
    ts_source::FileIdMap,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;

type Update = (PathBuf, ApplicationKey, super::super::Span, Arc<Recovered>);
type Failure = (
    PathBuf,
    ApplicationKey,
    super::super::Span,
    FailedSqlSources,
);

pub(super) fn apply(
    files: &mut FileIdMap<CheckFileFacts>,
    updates: Vec<Update>,
    failures: Vec<Failure>,
) {
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
    for (path, application, span, failures) in failures {
        let Some(file) = files.get_mut(&path) else {
            continue;
        };
        file.sql_source_prerequisites.record(
            &SqlSourceKey::application(
                SqlProjection::LowSyntax,
                &application.profile,
                &application.rule,
                application.index,
            ),
            span,
            failures,
        );
    }
}

pub(super) fn push_failures(
    out: &mut Vec<Failure>,
    owner: &Path,
    application: &ApplicationKey,
    span: super::super::Span,
    failures: FailedSqlSources,
) {
    if !failures.is_empty() {
        out.push((owner.to_path_buf(), application.clone(), span, failures));
    }
}

pub(super) fn finite_template_candidate(input: &Input) -> bool {
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
    first.is_some_and(|first| !text[..first].contains(';'))
}

#[cfg(test)]
mod tests;
