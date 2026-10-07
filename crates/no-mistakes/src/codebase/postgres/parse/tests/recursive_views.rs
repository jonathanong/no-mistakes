use super::super::{parse_postgres_sql, parse_postgres_sql_lenient};
use sqlparser::ast::Statement;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres-facts/source")
            .join(name),
    )
    .unwrap()
}

#[test]
fn strict_recursive_view_asts_preserve_the_implicit_binding_and_column_aliases() {
    let sql = fixture("recursive-views.sql");
    let statements = parse_postgres_sql(&sql).unwrap();
    for (index, name, column) in [
        (0, "recursive_view", "x"),
        (1, "RecursiveView", "X"),
        (2, "commented", "x"),
    ] {
        let Statement::CreateView(view) = &statements[index] else {
            panic!("expected view")
        };
        let with = view.query.with.as_ref().unwrap();
        assert!(with.recursive);
        assert_eq!(with.cte_tables.len(), 1);
        assert_eq!(with.cte_tables[0].alias.name.value, name);
        assert_eq!(with.cte_tables[0].alias.columns[0].name.value, column);
        assert!(with.cte_tables[0].query.to_string().starts_with("SELECT"));
    }
    // Ordinary/materialized view ASTs do not gain the implicit recursive wrapper.
    for statement in &statements[3..6] {
        let Statement::CreateView(view) = statement else {
            panic!("expected view")
        };
        assert!(view.query.with.is_none());
    }
}

#[test]
fn lenient_recursive_views_keep_scope_and_validate_columns_amid_unsupported_neighbors() {
    let sql = fixture("recursive-view-lenient.sql");
    assert!(parse_postgres_sql(&sql).is_err());
    let statements = parse_postgres_sql_lenient(&sql);
    assert_eq!(statements.len(), 5);
    for (statement, expected) in statements
        .iter()
        .zip(["nums", "Nums", "nested", "replaced"])
    {
        let Statement::CreateView(view) = statement else {
            panic!("expected view")
        };
        assert!(view.query.with.as_ref().unwrap().recursive);
        assert_eq!(
            view.query.with.as_ref().unwrap().cte_tables[0]
                .alias
                .name
                .value,
            expected
        );
    }
    assert!(matches!(&statements[4], Statement::Query(_)));
}

#[test]
fn recursive_statement_facts_exclude_self_reads_in_strict_and_lenient_paths() {
    let sql = fixture("recursive-view-lenient.sql");
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(facts.parse_failed);
    let names = facts
        .selects
        .iter()
        .flat_map(|select| &select.tables)
        .map(|table| table.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["app.seed", "app.Nums", "app.seed"]);
    let strict = &sql
        [sql.find("CREATE RECURSIVE").unwrap()..sql.find("CREATE RECURSIVE VIEW missing").unwrap()];
    let facts = crate::codebase::postgres::extract_sql_statement_facts(strict);
    assert!(!facts.parse_failed);
    assert_eq!(
        facts
            .selects
            .iter()
            .flat_map(|select| &select.tables)
            .map(|table| table.as_str())
            .collect::<Vec<_>>(),
        ["app.seed", "app.Nums"]
    );
}
