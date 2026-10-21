use std::collections::{BTreeMap, BTreeSet};

use crate::codebase::postgres::{
    SqlColumnClause, SqlColumnUseFact, SqlStatementFileFacts, SqlVerdictStatus,
};

#[test]
fn a_same_named_column_on_another_table_is_clear() {
    let tracked = [super::Tracked {
        table: "orders".to_string(),
        column: "total".to_string(),
        source: "id".to_string(),
        function: None,
    }];
    let mut columns = BTreeMap::new();
    for table in ["orders", "invoices"] {
        columns.insert(
            table.to_string(),
            BTreeSet::from(["id".to_string(), "total".to_string()]),
        );
    }
    let statement = SqlStatementFileFacts {
        mutation_column_uses: vec![SqlColumnUseFact {
            table: "invoices".to_string(),
            column: "total".to_string(),
            candidate_tables: None,
            clause: SqlColumnClause::Where,
            line: 3,
        }],
        ..Default::default()
    };
    let judgment = super::judge("where", &statement, &tracked, &columns);
    assert_eq!(judgment.status, SqlVerdictStatus::ProvenClear);
    assert!(judgment.reason.is_none());
}
