//! Shared literal validation and borrowing; each policy retains its specialized SQL parser.
use super::{Input, Span};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::SqlStatementFileFacts;
use std::{collections::BTreeMap, path::Path};

mod recovered;
pub(crate) use recovered::recovered_statements;

type Literal<T> = Result<Option<Vec<T>>, ()>;
#[derive(Clone)]
pub(crate) enum ProjectedLiteral<T> {
    Direct(Literal<T>),
    Recovered {
        alternatives: Vec<(crate::codebase::postgres::RecoveredSql, Result<Vec<T>, ()>)>,
    },
}
pub(crate) struct LiteralProjection<T> {
    pub project: fn(&SqlStatementFileFacts) -> Vec<T>,
    pub parse: fn(&str) -> Result<Vec<T>, ()>,
    pub rebase: fn(&mut T, usize),
    pub position: fn(&T) -> (usize, usize),
    pub parse_metric: &'static str,
}

pub(crate) fn project_literals<T: Clone>(
    file: &CheckFileFacts,
    facts: &CheckFactMap,
    path: &Path,
    source: &str,
    projection: LiteralProjection<T>,
) -> BTreeMap<(Option<super::helper_recovery::ApplicationKey>, Span), ProjectedLiteral<T>> {
    let observer = crate::diagnostics::current();
    // Borrow a max consumer's completed SQL projection; never evaluate it here.
    let mut prepared = BTreeMap::new();
    for (profile, embedded) in &file.embedded_sql {
        let Ok(statements) = facts.postgres_statements(path, Some(profile)) else {
            continue;
        };
        let mut statements = statements.iter();
        let mut calls = BTreeMap::new();
        let projected = embedded
            .calls
            .iter()
            .zip(&embedded.call_spans)
            .flat_map(|(call, span)| {
                call.statement_calls().filter_map(move |version| {
                    version
                        .sql_text
                        .as_deref()
                        .map(|sql| (call, *span, sql.to_string()))
                })
            });
        // Consume every alternative before deciding whether a call is exact.
        for ((call, span, sql), statement) in projected.zip(statements.by_ref()) {
            if call.variants.is_empty() {
                calls.insert(
                    span,
                    (
                        sql,
                        if statement.parse_failed {
                            Err(())
                        } else {
                            Ok(Some((projection.project)(statement)))
                        },
                    ),
                );
            }
        }
        prepared.insert(profile, calls);
    }
    let mut direct = BTreeMap::new();
    for (profile, calls) in &file.sql_identifier_syntax.profiles {
        for call in calls {
            let Input::Sql {
                source_span,
                text,
                holes,
                parameterized,
                positions,
            } = &call.input
            else {
                continue;
            };
            direct
                .entry((profile.clone(), *source_span))
                .or_insert_with(|| {
                    ProjectedLiteral::Direct((|| {
                        if !holes.is_empty()
                            || (parameterized.is_none()
                                && source
                                    .get(source_span.0 as usize + 1..source_span.1 as usize - 1)
                                    .is_none_or(|raw| !literal_matches(raw, text)))
                        {
                            return Ok(None);
                        }
                        // Low tags carry actual binds and physical positions; max
                        // may publish identifier markers at the same call span.
                        if parameterized.is_none() {
                            if let Some(facts) = prepared
                                .get(profile)
                                .and_then(|calls| calls.get(&call.span))
                            {
                                if facts.0 == *text {
                                    return facts.1.clone();
                                }
                            }
                        }
                        if let Some(observer) = &observer {
                            observer.increment(projection.parse_metric, 1);
                        }
                        let mut facts = (projection.parse)(text)?;
                        let base = crate::codebase::ts_source::byte_offset_to_line(
                            source,
                            source_span.0 as usize,
                        ) as usize;
                        for fact in &mut facts {
                            let (line, column) = (projection.position)(fact);
                            let origin = positions.iter().rev().find(|position| {
                                (position.sql_line as usize, position.sql_column as usize)
                                    <= (line, column)
                            });
                            let source_line = origin.map_or(base + line - 1, |position| {
                                position.source_line as usize + line - position.sql_line as usize
                            });
                            (projection.rebase)(fact, source_line - line + 1);
                        }
                        Ok(Some(facts))
                    })())
                });
        }
    }
    let mut out = BTreeMap::new();
    for ((_, span), literal) in direct {
        out.insert((None, span), literal);
    }
    let recovered = recovered_statements(file);
    for application in file.sql_identifier_syntax.application_keys() {
        let Some(calls) = file.sql_identifier_syntax.calls_for(&application) else {
            continue;
        };
        for call in calls {
            match &call.input {
                Input::Sql { .. } => {}
                Input::Recovered(proof) => {
                    let statements = recovered.get(&(application.clone(), call.span));
                    let alternatives = proof
                        .alternatives
                        .iter()
                        .enumerate()
                        .map(|(index, sql)| {
                            let facts = statements.and_then(|items| items.get(index)).ok_or(());
                            (
                                sql.clone(),
                                facts.and_then(|facts| {
                                    facts
                                        .as_ref()
                                        .map(|facts| (projection.project)(facts))
                                        .map_err(|_| ())
                                }),
                            )
                        })
                        .collect();
                    out.insert(
                        (Some(application.clone()), call.span),
                        ProjectedLiteral::Recovered { alternatives },
                    );
                }
                _ => {}
            }
        }
    }
    out
}
pub(super) fn literal_matches(raw: &str, cooked: &str) -> bool {
    // JavaScript normalizes physical CRLF template endings without decoding escapes.
    raw == cooked || (raw.contains('\r') && raw.replace("\r\n", "\n") == cooked)
}
