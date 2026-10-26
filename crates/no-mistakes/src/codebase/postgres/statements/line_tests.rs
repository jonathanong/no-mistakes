use super::extract_sql_statement_facts;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/statements")
            .join(name),
    )
    .unwrap()
}

#[test]
fn repeated_selects_use_their_own_relation_spans() {
    let facts = extract_sql_statement_facts(&fixture("own-statement-lines.sql"));
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .selects
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [2, 5, 6, 8, 10, 12, 14]
    );
}

#[test]
fn skipped_insert_plans_do_not_shift_executed_insert_locations() {
    let facts = extract_sql_statement_facts(&fixture("own-insert-lines.sql"));
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .inserts
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [3, 7, 9, 11, 13, 15]
    );
}

#[test]
fn prepared_insert_heads_do_not_borrow_skipped_or_nested_identity_policies() {
    let sql = fixture("own-insert-policy-lines.sql");
    let ast_sql = fixture("own-insert-lines.sql");
    // OVERRIDING is outside the parser grammar; the saved AST input uses equal-width spaces.
    let prepared = crate::codebase::postgres::parse::PreparedSql::new(&sql);
    let statements = crate::codebase::postgres::parse::parse_postgres_sql(&ast_sql).unwrap();
    let facts = super::facts::extract_from_parsed_with_recovered_placeholders(
        &sql,
        &prepared,
        &statements,
        false,
        false,
        None,
        Default::default(),
    );
    // The ignored INSERT must not transfer its policy; executed lines 7 and 13 override.
    assert!(!facts.inserts[0].assignments.is_empty());
    assert!(facts.inserts[1].assignments.is_empty());
    assert!(!facts.inserts[2].assignments.is_empty());
    assert!(!facts.inserts[3].assignments.is_empty());
    assert!(facts.inserts[4].assignments.is_empty());
    assert!(!facts.inserts[5].assignments.is_empty());
}

#[test]
fn repeated_triggers_use_create_before_their_own_ast_names() {
    let facts = extract_sql_statement_facts(&fixture("own-trigger-lines.sql"));
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .triggers
            .iter()
            .map(|fact| fact.line)
            .collect::<Vec<_>>(),
        [2, 4, 7]
    );
}

#[test]
fn multiline_select_locations_keep_the_own_from_relation_anchor() {
    let sql = fixture("own-statement-lines.sql");
    let facts = super::facts::extract_sql_variant_statement_facts(&sql, false, &[]);
    let locations = facts.variant_locations.as_ref().unwrap();
    let position = locations.position(super::SqlFactSite::Select(1)).unwrap();
    assert_eq!((position.sql_line, position.sql_column), (5, 6));
    let relation = locations
        .position(super::SqlFactSite::Relation(1, 0))
        .unwrap();
    assert_eq!((relation.sql_line, relation.sql_column), (5, 6));
}
