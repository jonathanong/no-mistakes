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

#[test]
fn lexical_positions_ignore_nested_comments_multiline_quotes_and_dollar_bodies() {
    let sql = fixture("lexical-keyword-locations.sql");
    assert_eq!(
        super::lines::last_word_position(&sql, "select", sql.len()),
        Some((16, 3))
    );
    assert_eq!(
        super::lines::last_word_position(&sql, "insert", sql.len()),
        Some((14, 5))
    );
    assert_eq!(
        super::lines::last_word_position(&sql, "missing", sql.len()),
        None
    );
    assert_eq!(
        super::lines::last_word_position(&sql, "select", sql.len() + 1),
        None
    );
}

#[test]
fn lexical_recovery_does_not_reinterpret_unterminated_text_as_statements() {
    for name in [
        "lexical-unclosed-block.sql",
        "lexical-unclosed-dollar.sql",
        "lexical-unclosed-quote.sql",
    ] {
        let sql = fixture(name);
        assert_eq!(
            super::lines::last_word_position(&sql, "select", sql.len()),
            Some((1, 1)),
            "{name}"
        );
        assert_eq!(
            super::lines::last_word_position(&sql, "insert", sql.len()),
            None,
            "{name}"
        );
    }
    let sql = fixture("lexical-dollar-bind.sql");
    assert_eq!(
        super::lines::last_word_position(&sql, "select", sql.len()),
        Some((2, 3))
    );
    let unicode = sql.find('é').unwrap() + 1;
    assert_eq!(
        super::lines::last_word_position(&sql, "select", unicode),
        None
    );
}

#[test]
fn fallback_insert_sources_match_only_real_heads_and_keep_columns() {
    let sql = fixture("lexical-keyword-locations.sql");
    let sources = super::lines::InsertSources::new(&sql);
    let parsed = super::super::parse::parse_postgres_sql(&fixture("own-insert-lines.sql")).unwrap();
    let insert = parsed
        .iter()
        .find_map(|statement| match statement {
            sqlparser::ast::Statement::Insert(insert) => Some(insert),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (sources.line(insert, 1), sources.column(insert, 1)),
        (13, 3)
    );
    assert_eq!(
        (sources.line(insert, 2), sources.column(insert, 2)),
        (14, 5)
    );
    assert_eq!((sources.line(insert, 9), sources.column(insert, 9)), (1, 1));
    assert!(sources
        .for_insert(insert, 1)
        .starts_with("INSERT /* separator */ INTO real_table"));
    assert!(sources
        .source(2)
        .starts_with("INSERT INTO real_table VALUES (2)"));
}
