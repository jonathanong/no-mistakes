use super::super::history::Catalogs;
use super::judge;
use crate::codebase::postgres::{
    GeneratedTable, GeneratedTableColumns, SqlBranchJudgment, SqlStatementFileFacts,
    SqlVerdictStatus, SqlWriteColumns, SqlWriteFact,
};
use std::collections::BTreeMap;

fn catalogs(order: Option<Vec<String>>, complete: bool) -> Catalogs {
    let mut columns = GeneratedTableColumns::default();
    columns.insert_table(GeneratedTable {
        name: "votes".to_string(),
        generated: ["created_at".to_string()].into_iter().collect(),
        column_order: order,
    });
    let mut known_tables = BTreeMap::new();
    if complete {
        known_tables.insert("votes".to_string(), true);
    }
    Catalogs {
        catalog: columns.clone(),
        trigger: GeneratedTableColumns::default(),
        combined: columns,
        known_tables,
    }
}

fn judgment(order: Option<Vec<String>>, complete: bool, width: Option<usize>) -> SqlBranchJudgment {
    let catalogs = catalogs(order, complete);
    let mut statement = SqlStatementFileFacts::default();
    statement.writes.push(SqlWriteFact {
        table: "votes".to_string(),
        columns: SqlWriteColumns::Positional(width),
        line: 1,
        column: 1,
        column_keys: None,
    });
    judge("generated-write", &statement, &|_, _| &catalogs)
}

#[test]
fn absent_width_abstains_and_a_counted_zero_clears() {
    let order = Some(vec!["id".to_string(), "created_at".to_string()]);
    for complete in [true, false] {
        let absent = judgment(None, complete, None);
        assert_eq!(absent.status, SqlVerdictStatus::Abstain, "{complete}");
        assert_eq!(absent.reason, Some("unknown-relation-or-write-width"));
        let zero = judgment(None, complete, Some(0));
        assert_eq!(zero.status, SqlVerdictStatus::ProvenClear, "{complete}");
        assert_eq!(zero.reason, None);
        let counted = judgment(None, complete, Some(2));
        assert_eq!(counted.status, SqlVerdictStatus::Abstain, "{complete}");
        assert_eq!(counted.reason, Some("unknown-relation-or-write-width"));
        let defaults = judgment(order.clone(), complete, None);
        assert_eq!(defaults.status, SqlVerdictStatus::Abstain, "{complete}");
        assert_eq!(defaults.reason, Some("unknown-relation-or-write-width"));
    }
    let hit = judgment(order, true, Some(2));
    assert_eq!(hit.status, SqlVerdictStatus::Violation);
    assert!(hit.survives_unknown);
    let covered = judgment(
        Some(vec!["id".to_string(), "created_at".to_string()]),
        true,
        Some(usize::MAX),
    );
    assert_eq!(covered.status, SqlVerdictStatus::Violation);
}
