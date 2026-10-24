//! Shared literal validation and borrowing; each policy retains its specialized SQL parser.
use super::{Input, Span};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::SqlStatementFileFacts;
use std::collections::BTreeMap;
use std::path::Path;

type Literal<T> = Result<Option<Vec<T>>, ()>;
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
) -> BTreeMap<Span, Literal<T>> {
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
                call.statement_calls()
                    .filter(|version| version.sql_text.is_some())
                    .map(move |_| (call, span))
            });
        // Consume every alternative before deciding whether a call is exact.
        for ((call, span), statement) in projected.zip(statements.by_ref()) {
            if call.variants.is_empty() {
                calls.insert(
                    *span,
                    if statement.parse_failed {
                        Err(())
                    } else {
                        Ok(Some((projection.project)(statement)))
                    },
                );
            }
        }
        prepared.insert(profile, calls);
    }
    let mut out = BTreeMap::new();
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
            out.entry(*source_span).or_insert_with(|| {
                if !holes.is_empty()
                    || (parameterized.is_none()
                        && source
                            .get(source_span.0 as usize + 1..source_span.1 as usize - 1)
                            .is_none_or(|raw| !literal_matches(raw, text)))
                {
                    return Ok(None);
                }
                if let Some(facts) = prepared
                    .get(profile)
                    .and_then(|calls| calls.get(&call.span))
                {
                    return facts.clone();
                }
                if let Some(observer) = &observer {
                    observer.increment(projection.parse_metric, 1);
                }
                let mut facts = (projection.parse)(text)?;
                let base =
                    crate::codebase::ts_source::byte_offset_to_line(source, source_span.0 as usize)
                        as usize;
                for fact in &mut facts {
                    let (line, column) = (projection.position)(fact);
                    let origin = positions.iter().rev().find(|position| {
                        (position.sql_line as usize, position.sql_column as usize) <= (line, column)
                    });
                    let source_line = origin.map_or(base + line - 1, |position| {
                        position.source_line as usize + line - position.sql_line as usize
                    });
                    (projection.rebase)(fact, source_line - line + 1);
                }
                Ok(Some(facts))
            });
        }
    }
    out
}
pub(super) fn literal_matches(raw: &str, cooked: &str) -> bool {
    // JavaScript normalizes physical CRLF template endings without decoding escapes.
    raw == cooked || (raw.contains('\r') && raw.replace("\r\n", "\n") == cooked)
}
