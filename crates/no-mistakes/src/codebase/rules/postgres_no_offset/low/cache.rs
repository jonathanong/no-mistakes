use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::sql_identifier_syntax::{Input, OffsetLiteral};
use std::collections::BTreeMap;
use std::path::Path;

type Span = (u32, u32);

pub(super) fn literals<'a>(
    file: &'a CheckFileFacts,
    facts: &CheckFactMap,
    path: &Path,
    source: &str,
) -> &'a BTreeMap<Span, OffsetLiteral> {
    let syntax = &file.sql_identifier_syntax;
    syntax.offset_literals.get_or_init(|| {
        let observer = crate::diagnostics::current();
        // Borrow a max consumer's completed SQL projection; never evaluate it here.
        let mut prepared = BTreeMap::new();
        for (profile, embedded) in &file.embedded_sql {
            let Ok(statements) = facts.postgres_statements(path, Some(profile)) else {
                continue;
            };
            let mut statements = statements.iter();
            let mut calls = BTreeMap::new();
            let projected =
                embedded
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
                            Ok(Some(statement.offset_uses.clone()))
                        },
                    );
                }
            }
            prepared.insert(profile, calls);
        }
        let mut out = BTreeMap::new();
        for (profile, calls) in &syntax.profiles {
            for call in calls {
                let Input::Sql {
                    source_span,
                    text,
                    holes,
                } = &call.input
                else {
                    continue;
                };
                out.entry(*source_span).or_insert_with(|| {
                    if !holes.is_empty()
                        || source
                            .get(source_span.0 as usize + 1..source_span.1 as usize - 1)
                            .is_none_or(|raw| raw != text)
                    {
                        return Ok(None);
                    }
                    if let Some(offsets) = prepared
                        .get(profile)
                        .and_then(|calls| calls.get(&call.span))
                    {
                        return offsets.clone();
                    }
                    if let Some(observer) = &observer {
                        observer.increment("postgres.no_offset.low.literal_parses", 1);
                    }
                    let mut offsets =
                        crate::codebase::postgres::sql_offset_facts(text).map_err(|_| ())?;
                    let base = crate::codebase::ts_source::byte_offset_to_line(
                        source,
                        source_span.0 as usize,
                    ) as usize;
                    for offset in &mut offsets {
                        offset.line += base - 1;
                    }
                    Ok(Some(offsets))
                });
            }
        }
        out
    })
}
