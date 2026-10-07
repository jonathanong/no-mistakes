//! Unsupported source occurrences remain explicit within their body owner.
use super::types::*;

pub(super) fn unsupported(statements: &[PostgresSqlStatement]) -> Vec<PostgresSqlDiagnostic> {
    let mut diagnostics = Vec::new();
    collect(statements, &mut diagnostics);
    diagnostics
}

fn collect(statements: &[PostgresSqlStatement], diagnostics: &mut Vec<PostgresSqlDiagnostic>) {
    for statement in statements {
        match &statement.facts {
            PostgresSqlStatementKind::Conditional { branches } => {
                for branch in branches {
                    collect(&branch.statements, diagnostics);
                }
            }
            // A nested DO block owns its own diagnostics and completeness.
            PostgresSqlStatementKind::DoBlock { .. } => {}
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
