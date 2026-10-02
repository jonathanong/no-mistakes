use super::constraint_name;
use sqlparser::ast::{Ident, IndexConstraint, KeyOrIndexDisplay, Statement, TableConstraint};

#[test]
fn constraint_name_reads_variants_postgres_sql_does_not_parse() {
    let index = TableConstraint::Index(IndexConstraint {
        display_as_key: false,
        name: Some(Ident::new("idx")),
        index_type: None,
        columns: vec![],
        index_options: vec![],
    });
    let fulltext =
        TableConstraint::FulltextOrSpatial(sqlparser::ast::FullTextOrSpatialConstraint {
            fulltext: true,
            index_type_display: KeyOrIndexDisplay::None,
            opt_index_name: Some(Ident::new("ft")),
            columns: vec![],
        });
    let exclude = TableConstraint::Exclude(sqlparser::ast::ExcludeConstraint {
        name: Some(Ident::new("ex")),
        index_method: None,
        elements: vec![],
        include: vec![],
        where_clause: None,
        characteristics: None,
    });
    let primary = TableConstraint::PrimaryKeyUsingIndex(sqlparser::ast::ConstraintUsingIndex {
        name: Some(Ident::new("pk")),
        index_name: Ident::new("existing"),
        characteristics: None,
    });
    let unique = TableConstraint::UniqueUsingIndex(sqlparser::ast::ConstraintUsingIndex {
        name: Some(Ident::new("uq")),
        index_name: Ident::new("existing"),
        characteristics: None,
    });
    assert_eq!(name_of(&index), Some("idx"));
    assert_eq!(name_of(&fulltext), Some("ft"));
    assert_eq!(name_of(&exclude), Some("ex"));
    assert_eq!(name_of(&primary), Some("pk"));
    assert_eq!(name_of(&unique), Some("uq"));
}

#[test]
fn declaration_line_skips_comments_and_quoted_text() {
    let facts = super::super::extract_migration_facts(
        "/* create table */\nSELECT 'create table';\nCREATE TABLE items (id int);",
    );
    let table = facts
        .declared_identifiers
        .iter()
        .find(|identifier| identifier.name == "items")
        .unwrap();
    assert_eq!(table.line, 3);
}

#[test]
fn unmatched_source_does_not_invent_a_declaration() {
    let statement = crate::codebase::postgres::parse_postgres_sql("CREATE TABLE items (id int)")
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let Statement::CreateTable(_) = &statement else {
        panic!("expected a table");
    };
    let mut locations = super::Locations::new("SELECT 1;");
    let names = super::collect(&mut locations, &statement);
    assert!(names.is_empty());
}

#[test]
fn procedure_names_drop_the_schema_and_unescape_quotes() {
    let facts = super::super::extract_migration_facts(
        "CREATE OR REPLACE PROCEDURE app.\"order\"\"proc\"() LANGUAGE plpgsql AS $$\nSELECT 1;\n$$;\n",
    );
    let names: Vec<_> = facts
        .declared_identifiers
        .iter()
        .map(|identifier| identifier.name.as_str())
        .collect();
    assert_eq!(names, ["order\"proc"]);
    assert_eq!(facts.declared_identifiers[0].line, 1);
}

#[test]
fn procedure_scan_skips_a_name_it_cannot_read() {
    let names =
        super::Locations::new("CREATE PROCEDURE (\nCREATE PROCEDURE \"open\nCREATE PROCEDURE   ")
            .unparsed_declarations();
    assert!(names.is_empty());
}

#[test]
fn unnamed_primary_key_adds_no_constraint_name() {
    let facts =
        super::super::extract_migration_facts("CREATE TABLE items (id int, PRIMARY KEY (id));");
    let names: Vec<_> = facts
        .declared_identifiers
        .iter()
        .map(|identifier| identifier.name.as_str())
        .collect();
    assert_eq!(names, ["items", "id"]);
}

#[test]
fn column_constraint_name_is_declared() {
    let facts = super::super::extract_migration_facts(
        "CREATE TABLE items (id int CONSTRAINT must_be_positive CHECK (id > 0));",
    );
    let names: Vec<_> = facts
        .declared_identifiers
        .iter()
        .map(|identifier| identifier.name.as_str())
        .collect();
    assert!(names.contains(&"must_be_positive"), "{names:?}");
}

#[test]
fn statements_without_declared_names_are_ignored() {
    let statement = crate::codebase::postgres::parse_postgres_sql("SELECT 1")
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let mut names = Vec::new();
    super::push_statement(&mut names, &statement, 1);
    assert!(names.is_empty());
}

#[test]
fn unterminated_comments_and_quotes_do_not_hide_a_later_name() {
    assert!(super::Locations::new("-- comment")
        .unparsed_declarations()
        .is_empty());
    assert!(super::Locations::new("/* comment")
        .unparsed_declarations()
        .is_empty());
    assert!(super::Locations::new("'unterminated")
        .unparsed_declarations()
        .is_empty());
}

fn name_of(constraint: &TableConstraint) -> Option<&str> {
    constraint_name(constraint).map(|name| name.value.as_str())
}

#[test]
fn invalid_identifier_tokens_fail_closed_without_inventing_names() {
    for fixture in [
        "missing.sql",
        "unicode-missing.sql",
        "nonword.sql",
        "qualifier.sql",
        "unicode-number.sql",
        "unicode-word.sql",
        "unicode-space.sql",
        "unicode-space-value.sql",
        "escape-word.sql",
        "escape-empty.sql",
        "escape-multiple.sql",
        "invalid-unicode.sql",
    ] {
        let tokens = fixture_tokens(fixture);
        assert!(
            super::procedures::identifier(&tokens, 2).is_none(),
            "{fixture}: {tokens:?}"
        );
    }
    for (fixture, expected) in [
        ("quoted-apostrophe.sql", "owner's_proc"),
        ("quoted-u.sql", "U"),
        ("plain-u.sql", "U"),
    ] {
        assert_eq!(
            super::procedures::identifier(&fixture_tokens(fixture), 2)
                .unwrap()
                .0,
            expected
        );
    }
    assert!(super::Locations::new(&fixture_sql("unknown-kind.sql"))
        .unparsed_declarations()
        .is_empty());
    let sql = fixture_sql("modifiers.sql");
    let facts = super::super::extract_migration_facts(&sql);
    assert!(facts
        .declared_identifiers
        .iter()
        .any(|name| name.name == "score"));
}

fn fixture_sql(name: &str) -> String {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/identifier-length/tokens")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

fn fixture_tokens(name: &str) -> Vec<sqlparser::tokenizer::TokenWithSpan> {
    crate::codebase::postgres::parse::unicode::tokenize_raw_unicode(&fixture_sql(name))
        .into_iter()
        .filter(|token| !matches!(token.token, sqlparser::tokenizer::Token::Whitespace(_)))
        .collect()
}

#[test]
fn recursive_views_and_language_prefixed_do_keep_declaration_locations() {
    let facts = super::super::extract_migration_facts(&fixture_sql("recursive-do.sql"));
    for (name, line) in [
        ("recursive_items", 1),
        ("recursive_column", 1),
        ("second_column", 1),
        ("recovered_items", 3),
        ("recovered_column", 3),
        ("recovered_proc", 4),
        ("view_without_columns", 7),
        ("malformed_columns", 8),
    ] {
        assert!(
            facts
                .declared_identifiers
                .iter()
                .any(|identifier| { identifier.name == name && identifier.line == line }),
            "missing {name}:{line}: {:?}",
            facts.declared_identifiers
        );
    }
}

#[test]
fn concurrently_is_a_table_name_and_only_an_index_modifier() {
    let facts = super::super::extract_migration_facts(&fixture_sql("keyword-name.sql"));
    for name in [
        "concurrently",
        "very_long_column_name",
        "keyword_constraint",
        "concurrently_index",
        "another_column",
    ] {
        assert!(
            facts
                .declared_identifiers
                .iter()
                .any(|identifier| identifier.name == name),
            "{name}: {facts:?}"
        );
    }
}
