use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::{SqlPolicyClause, SqlPolicyClauseUse};

pub(super) fn clause_target(clause: SqlPolicyClause) -> String {
    format!("clause:{}", clause.name())
}

pub(super) fn clause_finding(
    file: &str,
    used: &SqlPolicyClauseUse,
    opts: &CompiledOptions,
) -> Option<RuleFinding> {
    if !opts.clauses.contains(&used.clause) {
        return None;
    }
    let line = used.line.max(1);
    let message = match used.clause {
        SqlPolicyClause::CreateExtensionVersion => {
            "CREATE EXTENSION must omit VERSION so PostgreSQL can select an available default version"
        }
        SqlPolicyClause::DropCascade => {
            "SQL must omit CASCADE here. Handle dependent objects explicitly before running this statement"
        }
    };
    Some(RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(clause_target(used.clause)),
    })
}
