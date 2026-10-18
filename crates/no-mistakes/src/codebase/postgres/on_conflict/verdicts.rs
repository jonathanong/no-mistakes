use super::{arbiter, convergence, generated, trigger, Catalog};
use crate::codebase::postgres::statement_facts::{
    SqlInsertFact, SqlOnConflictAction, SqlStatementFileFacts,
};
use crate::codebase::postgres::{SqlBranchJudgment, SqlVerdictStatus};

/// Judge an enabled replay-safety obligation independently of finding priority.
pub(crate) fn judge_subcheck(
    check: &str,
    file: &SqlStatementFileFacts,
    catalog: &Catalog<'_>,
) -> SqlBranchJudgment {
    let mut unresolved = None;
    for insert in file.inserts.iter().filter(|insert| insert.executed) {
        let judgment = judge_insert(check, insert, catalog);
        if judgment.status == SqlVerdictStatus::Violation {
            return judgment;
        }
        if judgment.status == SqlVerdictStatus::Abstain {
            unresolved = unresolved.or(judgment.reason);
        }
    }
    if let Some(reason) = unresolved {
        SqlBranchJudgment::abstain(reason)
    } else {
        SqlBranchJudgment::clear()
    }
}

fn judge_insert(check: &str, insert: &SqlInsertFact, catalog: &Catalog<'_>) -> SqlBranchJudgment {
    if check == "replay-guard" {
        return if insert.guarded_select || insert.on_conflict.is_some() {
            SqlBranchJudgment::clear()
        } else {
            SqlBranchJudgment::violation(false)
        };
    }
    if check == "trigger-replay" && insert.guarded_select {
        if !known_table(insert, catalog) {
            return SqlBranchJudgment::abstain("unknown-schema-table");
        }
        return outcome(trigger::judge_guarded_select(insert, catalog));
    }
    if insert.guarded_select {
        return SqlBranchJudgment::clear();
    }
    let Some(conflict) = &insert.on_conflict else {
        return SqlBranchJudgment::clear();
    };
    if check == "trigger-replay" && conflict.action == SqlOnConflictAction::DoNothing {
        if !known_table(insert, catalog) {
            return SqlBranchJudgment::abstain("unknown-schema-table");
        }
        return outcome(trigger::judge(insert, conflict, catalog));
    }
    if conflict.action != SqlOnConflictAction::DoUpdate {
        return SqlBranchJudgment::clear();
    }
    let result = match check {
        "arbiter" => arbiter::judge(insert, conflict),
        "convergence" => convergence::judge(conflict, true, false),
        "volatility" => convergence::judge(conflict, false, true),
        "generated-arbiter" => {
            if !known_table(insert, catalog) {
                return SqlBranchJudgment::abstain("unknown-schema-table");
            }
            generated::judge(insert, conflict, catalog.schema)
        }
        "trigger-replay" => {
            if !known_table(insert, catalog) {
                return SqlBranchJudgment::abstain("unknown-schema-table");
            }
            trigger::judge(insert, conflict, catalog)
        }
        _ => return SqlBranchJudgment::abstain("unknown-enabled-subcheck"),
    };
    outcome(result)
}

fn known_table(insert: &SqlInsertFact, catalog: &Catalog<'_>) -> bool {
    catalog
        .schema
        .iter()
        .flat_map(|file| &file.tables)
        .any(|table| table.table_name.eq_ignore_ascii_case(&insert.table))
}

fn outcome(result: Option<String>) -> SqlBranchJudgment {
    match result {
        Some(message)
            if message.contains("cannot prove") || message.contains("missing from schema") =>
        {
            SqlBranchJudgment::abstain("unresolved-replay-safety")
        }
        Some(_) => SqlBranchJudgment::violation(false),
        None => SqlBranchJudgment::clear(),
    }
}
