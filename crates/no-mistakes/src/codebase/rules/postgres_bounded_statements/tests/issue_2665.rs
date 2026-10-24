use super::{catalog, offenders};
use crate::codebase::postgres::extract_sql_statement_facts;

#[test]
fn bounded_shapes_keep_source_cardinality_and_branch_correlation() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/issue-2665.sql"
    ));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog()))
        .map(|offender| (offender.table, offender.line))
        .collect();
    assert_eq!(
        found,
        [
            ("order_lines", 4),
            ("accounts", 8),
            ("orders", 11),
            ("accounts", 11),
            ("accounts", 14),
            ("accounts", 15),
            ("accounts", 16),
            ("accounts", 19),
            ("order_lines", 20),
            ("orders", 22),
        ]
        .map(|(name, line)| (name.to_owned(), line))
    );
}

#[test]
fn finite_output_rows_do_not_prove_array_elements_or_limit_values() {
    let sql = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/issue-2665-adversarial.sql"));
    let facts = extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let found: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog()))
        .map(|offender| (offender.table, offender.line))
        .collect();
    assert_eq!(
        found,
        [
            ("accounts", 2),
            ("accounts", 3),
            ("accounts", 4),
            ("accounts", 6),
            ("accounts", 8),
            ("accounts", 9),
            ("order_lines", 12),
            ("accounts", 17),
            ("accounts", 19),
            ("accounts", 21),
            ("accounts", 22),
            ("accounts", 24),
            ("accounts", 25),
            ("accounts", 26),
            ("accounts", 27),
            ("accounts", 29),
            ("accounts", 30),
            ("accounts", 33),
            ("accounts", 35),
            ("orders", 41),
            ("accounts", 41),
            ("orders", 43),
            ("accounts", 45),
            ("accounts", 49),
            ("accounts", 50),
            ("accounts", 51),
            ("accounts", 52),
            ("accounts", 53),
            ("accounts", 54),
            ("accounts", 55),
            ("accounts", 56),
            ("accounts", 57),
            ("accounts", 59),
            ("accounts", 60),
            ("accounts", 69),
            ("accounts", 70),
            ("accounts", 72),
            ("accounts", 76),
        ]
        .map(|(name, line)| (name.to_owned(), line))
    );
}

#[test]
fn shared_conjuncts_complete_different_composite_unique_keys_per_branch() {
    let root = super::fixture_root();
    let sources =
        crate::codebase::rules::source_store_for_files(&[root.join("schema-issue-2665-or.json")]);
    let catalog = crate::codebase::postgres::SchemaCatalog::load(
        &root,
        "schema-issue-2665-or.json",
        &sources,
    )
    .unwrap();
    let facts = extract_sql_statement_facts(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/issue-2665-composite-or.sql")));
    assert!(!facts.parse_failed);
    let lines: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog))
        .map(|offender| offender.line)
        .collect();
    assert_eq!(lines, [4, 5]);
}

#[test]
fn review_controls_preserve_guaranteed_rows_and_executed_input_evidence() {
    let facts = extract_sql_statement_facts(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/issue-2665-review.sql"
    )));
    assert!(!facts.parse_failed);
    let lines: Vec<_> = facts
        .bounds
        .iter()
        .flat_map(|fact| offenders(fact, &catalog()))
        .map(|offender| (offender.table, offender.line))
        .collect();
    assert_eq!(
        lines,
        [2, 3, 10, 11, 12, 14, 16, 17, 19, 20, 22, 24, 29, 31, 32]
            .map(|line| ("accounts".to_owned(), line))
    );
}
