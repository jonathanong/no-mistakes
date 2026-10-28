use super::parse_postgres_sql;
use crate::codebase::postgres::statements::extract_sql_statement_facts;
use sqlparser::ast::Statement;

const ACCEPTED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/analyze-list/accepted.sql"
));
const EXTENDED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/analyze-list/extended.sql"
));
const MALFORMED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/analyze-list/malformed.sql"
));
const COUNTEREXAMPLE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/analyze-list/counterexample.sql"
));
const UNSUPPORTED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../fixtures/postgres-facts/source/analyze-list/unsupported.sql"
));

#[test]
fn relation_lists_parse_as_all_their_analyze_targets() {
    let statements = parse_postgres_sql(ACCEPTED).expect("PostgreSQL ANALYZE relation lists");
    assert_eq!(statements.len(), 13);
    assert!(statements
        .iter()
        .all(|item| matches!(item, Statement::Analyze(_))));
    assert!(!extract_sql_statement_facts(ACCEPTED).parse_failed);
}

#[test]
fn option_list_is_not_reinterpreted_as_a_plain_relation_list() {
    assert!(parse_postgres_sql(UNSUPPORTED).is_err());
    assert!(extract_sql_statement_facts(UNSUPPORTED).parse_failed);
}

#[test]
fn malformed_relation_lists_keep_their_parse_failures() {
    for sql in MALFORMED.lines() {
        assert!(
            parse_postgres_sql(sql).is_err(),
            "accepted malformed SQL: {sql}"
        );
        assert!(extract_sql_statement_facts(sql).parse_failed, "{sql}");
    }
}

#[test]
fn following_mutation_remains_visible_after_a_valid_list() {
    let statements = parse_postgres_sql(COUNTEREXAMPLE).expect("two SQL statements");
    assert_eq!(statements.len(), 3);
    let facts = extract_sql_statement_facts(COUNTEREXAMPLE);
    assert!(!facts.parse_failed);
    assert!(facts
        .statement_kinds
        .iter()
        .any(|item| item.kind == "DROP TABLE"));
}

#[test]
fn verbose_and_column_lists_preserve_every_target_and_column() {
    let statements = parse_postgres_sql(EXTENDED).expect("PostgreSQL ANALYZE targets");
    let targets: Vec<_> = statements
        .iter()
        .map(|statement| {
            let Statement::Analyze(analyze) = statement else {
                panic!("expected ANALYZE")
            };
            (
                analyze.table_name.as_ref().map(ToString::to_string),
                analyze
                    .columns
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    // Exact targets catch rewrites that drop later relations or their columns.
    let expected = [
        (Some("a"), vec![]),
        (Some("public.b"), vec![]),
        (Some("a"), vec!["col"]),
        (Some("b"), vec![]),
        (Some("public.a"), vec!["first_col", "\"Mixed\""]),
        (Some("\"Schema\".\"Table\""), vec!["\"Quoted\""]),
        (Some("c"), vec!["last_col"]),
        (Some("\"VERBOSE\""), vec!["id"]),
        (Some("public.\"VERBOSE\""), vec![]),
        (Some("single_table"), vec!["id"]),
        (None, vec![]),
    ]
    .into_iter()
    .map(|(table, columns)| {
        (
            table.map(str::to_owned),
            columns.into_iter().map(str::to_owned).collect::<Vec<_>>(),
        )
    })
    .collect::<Vec<_>>();
    assert_eq!(targets, expected);
    assert!(!extract_sql_statement_facts(EXTENDED).parse_failed);
}
