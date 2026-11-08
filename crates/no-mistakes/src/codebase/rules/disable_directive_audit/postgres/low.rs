use super::*;
use crate::codebase::postgres::sql_identifier_syntax::{helper_recovery::ApplicationKey, Input};

pub(super) fn collect(
    comments: &mut Vec<PreparedComment>,
    scope: &AuditScope<'_>,
    facts: &CheckFactMap,
    path: &Path,
    key: &ApplicationKey,
) {
    let Some(file) = facts.ts.get(path) else {
        return;
    };
    let Some(calls) = file.sql_identifier_syntax.calls_for(key) else {
        return;
    };
    for call in calls {
        match &call.input {
            Input::Sql {
                text,
                positions,
                source_span,
                ..
            } if has_directive(text) => {
                let Some(source) =
                    crate::codebase::rules::suppression::read_directive_source(scope.sources, path)
                else {
                    continue;
                };
                locations::collect(
                    comments,
                    scope,
                    path,
                    SqlText {
                        sql: text,
                        positions,
                        base: crate::codebase::ts_source::byte_offset_to_line(
                            &source,
                            source_span.0 as usize,
                        ),
                    },
                );
            }
            Input::Recovered(proof) if !proof.nonexecuting => {
                for sql in &proof.alternatives {
                    locations::collect(
                        comments,
                        scope,
                        path,
                        SqlText {
                            sql: &sql.sql,
                            positions: &sql.positions,
                            base: sql.line,
                        },
                    );
                }
            }
            _ => {}
        }
    }
}
