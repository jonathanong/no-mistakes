use super::{owner, CompiledOptions, Tracked, RULE_ID};
use crate::codebase::check_facts::{CheckFactMap, PathMembership};
use crate::codebase::postgres::{
    record_sql_policy_verdicts, EmbeddedSqlFileFacts, SqlBranchJudgment, SqlColumnClause,
    SqlStatementFileFacts,
};
use crate::codebase::ts_source::relative_slash_path;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn record(
    root: &Path,
    opts: &CompiledOptions,
    embedded: &[&EmbeddedSqlFileFacts],
    queries: &PathMembership<'_>,
    tracked: &[Tracked],
    columns: &BTreeMap<String, BTreeSet<String>>,
    facts: &CheckFactMap,
) -> anyhow::Result<()> {
    let checks = [
        (opts.where_clause, "where"),
        (opts.join_clause, "join"),
        (opts.order_clause, "order-by"),
    ]
    .into_iter()
    .filter(|(enabled, _)| *enabled)
    .map(|(_, name)| name.to_string())
    .collect::<Vec<_>>();
    for file in embedded.iter().filter(|file| queries.contains(&file.path)) {
        record_sql_policy_verdicts(
            crate::codebase::postgres::SqlPolicyVerdictContext {
                facts,
                embedded: file,
                statements: facts.postgres_statements(&file.path, Some(&opts.embedded))?,
            },
            RULE_ID,
            &relative_slash_path(root, &file.path),
            &opts.embedded,
            &checks,
            opts.advise_unanalyzable,
            |check, _, statement| judge(check, statement, tracked, columns),
        );
    }
    Ok(())
}

fn judge(
    check: &str,
    statement: &SqlStatementFileFacts,
    tracked: &[Tracked],
    columns: &BTreeMap<String, BTreeSet<String>>,
) -> SqlBranchJudgment {
    let clause = match check {
        "where" => SqlColumnClause::Where,
        "join" => SqlColumnClause::Join,
        _ => SqlColumnClause::OrderBy,
    };
    let mut unknown_owner = false;
    for use_ in statement
        .selects
        .iter()
        .flat_map(|select| select.column_uses.iter())
        .chain(&statement.mutation_column_uses)
        .filter(|use_| use_.clause == clause)
    {
        if !tracked.iter().any(|item| item.column == use_.column) {
            continue;
        }
        let Some(table) = owner(use_, columns) else {
            unknown_owner = true;
            continue;
        };
        if !columns.contains_key(&table) {
            unknown_owner = true;
            continue;
        }
        if tracked
            .iter()
            .any(|item| item.table == table && item.column == use_.column)
        {
            return SqlBranchJudgment::violation(true);
        }
    }
    if unknown_owner {
        SqlBranchJudgment::abstain("unresolved-column-owner")
    } else {
        SqlBranchJudgment::clear()
    }
}
