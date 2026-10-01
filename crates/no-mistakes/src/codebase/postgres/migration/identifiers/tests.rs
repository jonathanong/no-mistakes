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
fn missing_opening_phrase_uses_line_one() {
    let statement = crate::codebase::postgres::parse_postgres_sql("CREATE TABLE items (id int)")
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let Statement::CreateTable(_) = &statement else {
        panic!("expected a table");
    };
    let mut from = 0;
    let names = super::collect("SELECT 1;", &statement, &mut from);
    assert_eq!(names[0].line, 1);
    assert_eq!(names[0].name, "items");
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

fn name_of(constraint: &TableConstraint) -> Option<&str> {
    constraint_name(constraint).map(|name| name.value.as_str())
}
