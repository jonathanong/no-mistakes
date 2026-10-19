//! Project conflict metadata from the AST owned by the shared statement pass.
use super::{collect::collect_statement, raw::raw_conflicts, SqlConflictInsertFact};
use sqlparser::ast::Statement;

pub(in crate::codebase::postgres) fn analyze_parsed(
    sql: &str,
    statements: &[Statement],
    binds: &[(u32, u32)],
) -> anyhow::Result<Vec<SqlConflictInsertFact>> {
    let mut raw = raw_conflicts(sql)?.into_iter();
    let mut inserts = Vec::new();
    for statement in statements {
        let mut executed = Vec::new();
        crate::codebase::postgres::statements::walk_executed(statement, &mut executed);
        for statement in executed {
            collect_statement(statement, &mut raw, binds, &mut inserts)?;
        }
    }
    anyhow::ensure!(
        raw.next().is_none(),
        "could not align ON CONFLICT clauses with INSERT statements"
    );
    Ok(inserts)
}

pub(in crate::codebase::postgres) struct ConflictRecovery {
    pub reliable: Vec<SqlConflictInsertFact>,
    pub failure_overlaps_conflict: bool,
}

/// Keep conflicts whose semicolon-delimited chunk parsed completely.
/// A failure inside that statement, or a CTE wrapped around it, is not reliable.
pub(in crate::codebase::postgres) fn reliable_conflicts(
    sql: &str,
    statements: &[Statement],
    binds: &[(u32, u32)],
    complete: &[bool],
) -> ConflictRecovery {
    let Ok(raw) = raw_conflicts(sql) else {
        return ConflictRecovery {
            reliable: Vec::new(),
            failure_overlaps_conflict: true,
        };
    };
    let mut raw = raw.into_iter();
    let mut reliable = Vec::new();
    let mut failure_overlaps_conflict = false;
    for (index, statement) in statements.iter().enumerate() {
        let mut local = Vec::new();
        let mut executed = Vec::new();
        crate::codebase::postgres::statements::walk_executed(statement, &mut executed);
        let mut failed = false;
        for statement in executed {
            if collect_statement(statement, &mut raw, binds, &mut local).is_err() {
                failed = true;
                break;
            }
        }
        let is_complete = complete.get(index).copied().unwrap_or(false);
        if is_complete && !failed {
            reliable.extend(local);
        } else if failed || !local.is_empty() {
            failure_overlaps_conflict = true;
        }
    }
    if raw.next().is_some() {
        failure_overlaps_conflict = true;
    }
    ConflictRecovery {
        reliable,
        failure_overlaps_conflict,
    }
}
