use crate::codebase::postgres::statements::{
    extract_sql_statement_facts_with_recovered_placeholders, extract_sql_variant_statement_facts,
    SqlFactSite,
};
use crate::codebase::postgres::SqlWriteColumns;

fn fixture(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn supported_write_forms_keep_assignment_and_clause_positions() {
    let sql = fixture("variants-provenance-write-forms.sql");
    let facts = extract_sql_variant_statement_facts(&sql, true, &[]);
    assert!(!facts.parse_failed);
    assert_eq!(facts.updates.len(), 2);
    assert_eq!(facts.writes.len(), 7);
    let locations = facts.variant_locations.as_ref().unwrap();
    assert_eq!(facts.updates[0].len(), 2);
    assert_eq!(facts.updates[1].len(), 2);
    assert!(matches!(
        facts.writes[2].columns,
        SqlWriteColumns::Positional(None)
    ));
    assert!(matches!(
        facts.writes[3].columns,
        SqlWriteColumns::Positional(Some(2))
    ));
    assert!(matches!(facts.writes[4].columns, SqlWriteColumns::All));
    assert!(matches!(facts.writes[5].columns, SqlWriteColumns::All));
    for (index, expected_line) in [(0, 2), (1, 4), (4, 8), (5, 9), (6, 12)] {
        let position = locations.position(SqlFactSite::Write(index)).unwrap();
        assert_eq!(position.sql_line, expected_line);
    }
    assert!(format!("{facts:#?}").contains("variant_locations"));
}

#[test]
fn malformed_setting_candidates_preserve_legacy_recovery_and_origins() {
    let sql = fixture("variants-provenance-recovery.sql");
    let facts = extract_sql_variant_statement_facts(&sql, false, &[]);
    assert!(facts.parse_failed);
    assert!(!facts.selects.is_empty());
    assert_eq!(facts.setting_uses.len(), 2);
    let locations = facts.variant_locations.as_ref().unwrap();
    for (index, name, line, column) in [(0, "app", 4, 5), (1, "statement_timeout", 6, 19)] {
        assert_eq!(facts.setting_uses[index].name, name);
        let position = locations.position(SqlFactSite::Setting(index)).unwrap();
        assert_eq!((position.sql_line, position.sql_column), (line, column));
    }
    let legacy = extract_sql_statement_facts_with_recovered_placeholders(&sql, false, &[]);
    let mut comparable = facts;
    comparable.variant_locations = None;
    assert_eq!(comparable, legacy);
}
