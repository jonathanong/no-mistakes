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
