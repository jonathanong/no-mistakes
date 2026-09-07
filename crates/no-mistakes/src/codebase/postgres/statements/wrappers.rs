use sqlparser::ast::Statement;

pub(super) fn walk_executed<'a>(statement: &'a Statement, out: &mut Vec<&'a Statement>) {
    match statement {
        Statement::Explain {
            analyze, statement, ..
        } => {
            if *analyze {
                walk_executed(statement, out);
            }
        }
        Statement::Prepare { statement, .. } => walk_executed(statement, out),
        Statement::CreateFunction(_) | Statement::CreateProcedure { .. } => {}
        other => out.push(other),
    }
}
