use super::{analyze_conflict_inserts, analyze_parsed, SqlConflictTarget};
use crate::codebase::postgres::parse::PreparedSql;

#[test]
fn prepared_conflict_projection_recovers_the_consumer_enum_insert() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-conflict-recovery/query.sql"
    ));
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-sql-verdicts/fixture/consumer-conflict-recovery/src/query.ts"
    ));
    assert!(source.contains(sql.trim_end()));

    // The historical exact-call reparse rejected this valid PostgreSQL form.
    assert!(analyze_conflict_inserts(sql).is_err());
    let prepared = PreparedSql::new(sql);
    let statements = prepared.parse_policy().unwrap();
    let inserts = analyze_parsed(sql, &statements, &[]).unwrap();
    assert_eq!(inserts.len(), 1);
    assert_eq!(inserts[0].target, SqlConflictTarget::Targetless);
    assert!(inserts[0].source.multi_row);
}
