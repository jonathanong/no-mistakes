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
mod tests;
