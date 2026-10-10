//! Unsupported source occurrences remain explicit within their body owner.
use super::types::*;

pub(super) fn utility_spans(
    occurrences: &[PostgresSqlProceduralOccurrence],
) -> super::completeness::UtilitySpans {
    fn collect(
        occurrences: &[PostgresSqlProceduralOccurrence],
        spans: &mut super::completeness::UtilitySpans,
    ) {
        for occurrence in occurrences {
            if occurrence.kind == PostgresSqlProceduralOccurrenceKind::Utility {
                spans.insert((occurrence.span.start.offset, occurrence.span.end.offset));
            }
            collect(&occurrence.occurrences, spans);
        }
    }

    let mut spans = super::completeness::UtilitySpans::new();
    collect(occurrences, &mut spans);
    spans
}

pub(super) fn unsupported(
    statements: &[PostgresSqlStatement],
    utilities: &super::completeness::UtilitySpans,
) -> Vec<PostgresSqlDiagnostic> {
    let mut diagnostics = Vec::new();
    collect(statements, utilities, &mut diagnostics);
    diagnostics
}

fn collect(
    statements: &[PostgresSqlStatement],
    utilities: &super::completeness::UtilitySpans,
    diagnostics: &mut Vec<PostgresSqlDiagnostic>,
) {
    for statement in statements {
        match &statement.facts {
            PostgresSqlStatementKind::Conditional { branches } => {
                for branch in branches {
                    collect(&branch.statements, utilities, diagnostics);
                }
            }
            PostgresSqlStatementKind::Wrapper { wrapper } => {
                collect(&wrapper.statements, utilities, diagnostics);
                diagnostics.extend(wrapper.diagnostics.iter().cloned());
            }
            PostgresSqlStatementKind::CreateFunction { function } => {
                collect(&function.wrapper.statements, utilities, diagnostics);
                diagnostics.extend(function.wrapper.diagnostics.iter().cloned());
            }
            // A nested DO block owns its own diagnostics and completeness.
            PostgresSqlStatementKind::DoBlock { .. } => {}
            PostgresSqlStatementKind::Other
                if utilities
                    .contains(&(statement.span.start.offset, statement.span.end.offset)) => {}
            facts if !super::completeness::statement(facts) => {
                let message = if matches!(facts, PostgresSqlStatementKind::Other) {
                    "Unsupported nested procedural statement; this source occurrence has no supported typed SQL facts"
                } else {
                    "Nested SQL facts are incomplete or unsupported; inspect the statement's typed facts"
                };
                diagnostics.push(PostgresSqlDiagnostic {
                    message: message.into(),
                    span: Some(statement.span.clone()),
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn span(start: usize, end: usize) -> PostgresSqlSpan {
        let position = |offset| PostgresSqlPosition {
            offset,
            line: 1,
            column: offset + 1,
        };
        PostgresSqlSpan {
            start: position(start),
            end: position(end),
        }
    }

    fn statement(
        facts: PostgresSqlStatementKind,
        start: usize,
        end: usize,
    ) -> PostgresSqlStatement {
        PostgresSqlStatement {
            ordinal: 0,
            span: span(start, end),
            sql: String::new(),
            facts,
        }
    }

    fn wrapper(child: PostgresSqlStatement) -> PostgresSqlWrapper {
        PostgresSqlWrapper {
            wrapper_kind: PostgresSqlWrapperKind::Explain,
            execution: PostgresSqlExecution::NonExecuting,
            statements: vec![child],
            span: None,
            complete: false,
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn classified_utility_spans_propagate_through_wrappers() {
        let child = statement(PostgresSqlStatementKind::Other, 10, 11);
        let spans = BTreeSet::from([(10, 11)]);
        let wrapped = statement(
            PostgresSqlStatementKind::Wrapper {
                wrapper: wrapper(child.clone()),
            },
            0,
            11,
        );
        let function = PostgresSqlFunction {
            wrapper: wrapper(child),
            name: PostgresSqlName {
                parts: Vec::new(),
                sql: String::new(),
            },
            arguments: Vec::new(),
            return_type: None,
            returns_set: false,
            language: None,
            behavior: None,
            body_sql: None,
            or_replace: false,
            temporary: false,
            called_on_null: None,
            parallel: None,
            configuration: Vec::new(),
            security: None,
        };
        let created = statement(PostgresSqlStatementKind::CreateFunction { function }, 0, 11);

        assert!(unsupported(&[wrapped, created], &spans).is_empty());
    }
}
