use super::history::Catalogs;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::dml::writes::positional_insert_hits;
use crate::codebase::postgres::{
    record_sql_policy_verdicts, EmbeddedSqlFileFacts, EmbeddedSqlOptions, SqlBranchJudgment,
    SqlStatementFileFacts, SqlWriteColumns,
};

pub(super) fn record<'a>(
    file: &str,
    profile: &EmbeddedSqlOptions,
    embedded: &EmbeddedSqlFileFacts,
    statements: &[SqlStatementFileFacts],
    lookup: &dyn Fn(usize) -> &'a Catalogs,
    include_trigger: bool,
    advisory: bool,
    facts: &CheckFactMap,
) {
    let mut checks = vec!["generated-write".to_string()];
    if include_trigger {
        checks.push("trigger-maintained-write".to_string());
    }
    record_sql_policy_verdicts(
        facts,
        super::super::RULE_ID,
        file,
        profile,
        embedded,
        statements,
        &checks,
        advisory,
        |check, _, statement| judge(check, statement, lookup),
    );
}

fn judge<'a>(
    check: &str,
    statement: &SqlStatementFileFacts,
    lookup: &dyn Fn(usize) -> &'a Catalogs,
) -> SqlBranchJudgment {
    let mut unknown = false;
    for write in &statement.writes {
        let snapshot = lookup(write.line);
        let catalog = if check == "generated-write" {
            &snapshot.catalog
        } else {
            &snapshot.trigger
        };
        let Some(meta) = catalog.get(&write.table) else {
            unknown |= !snapshot.known_complete(&write.table);
            continue;
        };
        let violation = match &write.columns {
            SqlWriteColumns::Named(columns) => columns
                .iter()
                .any(|column| meta.generated.contains(&column.to_ascii_lowercase())),
            SqlWriteColumns::Positional(width) => {
                if width.is_none() || meta.column_order.is_none() {
                    unknown = true;
                    false
                } else {
                    !positional_insert_hits(&write.table, meta, *width).is_empty()
                }
            }
            SqlWriteColumns::All => !meta.generated.is_empty(),
        };
        if violation {
            return SqlBranchJudgment::violation(true);
        }
        unknown |= !snapshot.known_complete(&write.table);
    }
    if unknown {
        SqlBranchJudgment::abstain("unknown-relation-or-write-width")
    } else {
        SqlBranchJudgment::clear()
    }
}
