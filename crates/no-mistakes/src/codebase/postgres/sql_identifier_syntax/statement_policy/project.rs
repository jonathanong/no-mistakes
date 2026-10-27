use super::super::{helper_recovery::ApplicationKey, Input};
use super::{calls, direct, Projection};
use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::postgres::{EmbeddedSqlCall, EmbeddedSqlKind, SqlStatementFileFacts};
use std::path::Path;

pub(super) fn build(
    file: &CheckFileFacts,
    facts: &CheckFactMap,
    path: &Path,
    key: &ApplicationKey,
) -> anyhow::Result<Projection> {
    if file.parse_error.is_some() || file.ts.parse_error.is_some() {
        return Ok(Projection {
            embedded: calls::empty(path),
            statements: Vec::new(),
        });
    }
    let source = file
        .source
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("{}: prepared low SQL source missing", key.rule))?;
    let syntax = file
        .sql_identifier_syntax
        .calls_for(key)
        .ok_or_else(|| anyhow::anyhow!("{}: prepared low SQL syntax missing", key.rule))?;
    let literals = direct::literals(file, facts, path, source);
    let recovered = super::super::literal_projection::recovered::raw_statements(file);
    let mut out = Projection {
        embedded: calls::empty(path),
        statements: Vec::new(),
    };
    for call in syntax {
        let index = out.embedded.calls.len();
        let first_statement = out.statements.len();
        let projected = match &call.input {
            Input::Sql {
                source_span,
                text,
                positions,
                ..
            } => {
                match literals
                    .get(&(key.profile.clone(), *source_span))
                    .and_then(Option::as_ref)
                {
                    Some(literal) => {
                        let base = crate::codebase::ts_source::byte_offset_to_line(
                            source,
                            source_span.0 as usize,
                        );
                        let projected = EmbeddedSqlCall {
                            line: call.line,
                            declaration_line: Some(base),
                            kind: EmbeddedSqlKind::ImmutableLocal,
                            sql_text: Some(text.clone()),
                            sql_source_positions: positions.clone(),
                            ..Default::default()
                        };
                        let mut statement = (*literal.facts).clone();
                        if !literal.mapped {
                            crate::codebase::postgres::collect::dml::rebase_embedded_lines(
                                &mut statement,
                                &projected,
                                false,
                            );
                        }
                        locate(&mut statement, path, call.line, index, 0);
                        out.statements.push(statement);
                        projected
                    }
                    None => calls::unknown(call),
                }
            }
            Input::Recovered(proof) => {
                let projected = calls::recovered(call, proof);
                if proof.nonexecuting && proof.complete() {
                    let mut statement = SqlStatementFileFacts::default();
                    locate(&mut statement, path, call.line, index, 0);
                    out.statements.push(statement);
                } else {
                    // The request cache records every recovered call and retains one
                    // fact payload per alternative, including partially invalid SQL.
                    let statements = recovered
                        .get(&(key.clone(), call.span))
                        .expect("recovered SQL calls have prepared statement alternatives");
                    for (variant_index, (version, statement)) in
                        projected.statement_calls().zip(statements).enumerate()
                    {
                        let mut statement = (**statement).clone();
                        crate::codebase::postgres::collect::dml::rebase_embedded_lines(
                            &mut statement,
                            &version,
                            true,
                        );
                        locate(&mut statement, path, call.line, index, variant_index);
                        out.statements.push(statement);
                    }
                }
                projected
            }
            _ => calls::unknown(call),
        };
        if matches!(call.input, Input::Recovered(_)) {
            let lines = calls::helper_lines(file, call);
            for statement in &mut out.statements[first_statement..] {
                if let Some(locations) = &mut statement.variant_locations {
                    locations.caller_lines = lines.clone();
                }
            }
            out.embedded.imported_call_lines.insert(call.span, lines);
        }
        out.embedded.calls.push(projected);
        out.embedded.call_spans.push(call.span);
    }
    Ok(out)
}
fn locate(
    statement: &mut SqlStatementFileFacts,
    path: &Path,
    line: u32,
    call: usize,
    variant: usize,
) {
    statement.path = path.to_path_buf();
    if let Some(locations) = &mut statement.variant_locations {
        locations.original_call_line = line;
        locations.call_index = call;
        locations.variant_index = variant;
    }
}
