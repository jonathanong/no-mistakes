use super::super::{clauses::clause_target, CompiledOptions};
use crate::codebase::postgres::{SqlBranchJudgment, SqlStatementFileFacts};

pub(super) fn checks(opts: &CompiledOptions) -> Vec<String> {
    let mut checks = opts
        .banned
        .iter()
        .map(|kind| format!("statement:{kind}"))
        .chain(opts.settings.iter().map(|name| format!("setting:{name}")))
        .chain(opts.clauses.iter().map(|clause| clause_target(*clause)))
        .collect::<Vec<_>>();
    checks.sort();
    checks
}

pub(super) fn judge(check: &str, statement: &SqlStatementFileFacts) -> SqlBranchJudgment {
    let violates = match check.split_once(':') {
        Some(("statement", kind)) => statement
            .statement_kinds
            .iter()
            .any(|item| item.kind == kind),
        Some(("setting", name)) => statement
            .setting_uses
            .iter()
            .any(|setting| setting.name == name),
        Some(("clause", name)) => statement
            .clause_uses
            .iter()
            .any(|used| used.clause.name() == name),
        _ => false,
    };
    if violates {
        SqlBranchJudgment::violation(true)
    } else {
        SqlBranchJudgment::clear()
    }
}
