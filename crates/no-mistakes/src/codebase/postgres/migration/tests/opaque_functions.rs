use super::extract_migration_facts;
use crate::codebase::postgres::{parse, SqlSchemaFileFacts};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-ddl-unanalyzable/fixture/opaque-functions/migrations")
            .join(name),
    )
    .unwrap()
}

#[test]
fn opaque_functions_keep_three_body_shapes_out_of_ddl_uncertainty() {
    for name in ["execute-using.sql", "execute-into.sql", "conflict-row.sql"] {
        let sql = fixture(name);
        let generic = extract_migration_facts(&sql);
        let ddl = generic.ddl_facts();
        assert!(ddl.unanalyzable_statements.is_empty(), "{name}: {ddl:?}");
        assert_eq!(ddl.add_columns.len(), 1, "{name}: {ddl:?}");
        assert_eq!(ddl.foreign_keys.len(), 1, "{name}: {ddl:?}");
        assert_eq!(ddl.unnamed_constraints.len(), 1, "{name}: {ddl:?}");
        assert_eq!(ddl.not_valid_constraints.len(), 1, "{name}: {ddl:?}");
        assert_eq!(ddl.indexes.len(), 2, "{name}: {ddl:?}");
        assert!(ddl
            .declared_identifiers
            .iter()
            .any(|name| name.name.len() > 63));
        if name.starts_with("execute-") {
            assert!(!generic.unanalyzable_statements.is_empty(), "{generic:?}");
        }
    }
}

#[test]
fn opaque_functions_preserve_body_inspection_and_executable_do_ddl() {
    let generic = extract_migration_facts(&fixture("body-ddl.sql"));
    let ddl = generic.ddl_facts();
    assert_eq!(
        generic
            .add_columns
            .iter()
            .map(|column| column.column_name.as_str())
            .collect::<Vec<_>>(),
        ["reported", "ignored", "dynamic_ignored"]
    );
    assert_eq!(
        ddl.add_columns
            .iter()
            .map(|column| column.column_name.as_str())
            .collect::<Vec<_>>(),
        ["reported"]
    );
    assert!(generic
        .statement_kinds
        .iter()
        .any(|kind| kind.kind == "TRUNCATE"));
    assert!(!ddl
        .statement_kinds
        .iter()
        .any(|kind| kind.kind == "TRUNCATE"));
    assert!(ddl.unanalyzable_statements.is_empty(), "{ddl:?}");
}

#[test]
fn opaque_functions_top_level_conflict_remains_analyzable() {
    let sql = fixture("top-level-conflict.sql");
    let located = parse::parse_postgres_sql_with_function_sources(&sql);
    assert!(
        super::super::recovery_failures(&located).is_empty(),
        "{located:?}"
    );
    let prepared = parse::PreparedSql::new(&sql);
    assert!(prepared.parse_policy().is_ok());
    let facts = extract_migration_facts(&sql);
    assert!(facts.unanalyzable_statements.is_empty(), "{facts:?}");
}

#[test]
fn opaque_functions_manual_facts_retain_existing_projection() {
    let facts = SqlSchemaFileFacts::default();
    assert!(std::ptr::eq(&facts, facts.ddl_facts()));
}

#[test]
fn opaque_functions_nested_in_do_keep_header_names_and_hide_body_names() {
    let generic = extract_migration_facts(&fixture("nested-stored.sql"));
    let ddl = generic.ddl_facts();
    assert!(generic
        .add_columns
        .iter()
        .any(|column| column.column_name == "ignored"));
    assert_eq!(
        ddl.add_columns
            .iter()
            .map(|column| column.column_name.as_str())
            .collect::<Vec<_>>(),
        ["reported"]
    );
    let long_names: Vec<_> = ddl
        .declared_identifiers
        .iter()
        .filter(|name| name.name.len() > 63)
        .map(|name| name.name.as_str())
        .collect();
    assert_eq!(
        long_names,
        ["this_function_name_is_longer_than_the_postgres_identifier_limit_of_sixty_three_bytes"]
    );
    assert!(ddl.unanalyzable_statements.is_empty(), "{ddl:?}");
}

#[test]
fn opaque_functions_conflict_recovery_keeps_predicates_and_rejects_trailing_sql() {
    use crate::codebase::postgres::source::*;
    let facts = parse_postgres_source(&PostgresSqlSource {
        sql: fixture("top-level-conflict.sql"),
        file_name: None,
    });
    assert!(facts.diagnostics.is_empty(), "{facts:?}");
    let PostgresSqlStatementKind::Insert { insert } = &facts.statements[0].facts else {
        panic!()
    };
    let conflict = insert.on_conflict.as_ref().unwrap();
    assert_eq!(conflict.predicate.as_ref().unwrap().sql, "col IS NOT NULL");
    let PostgresSqlConflictAction::DoUpdate { predicate, .. } = &conflict.action else {
        panic!()
    };
    assert_eq!(
        predicate.as_ref().unwrap().sql,
        "(items.a, items.b) IS DISTINCT FROM (EXCLUDED.a, EXCLUDED.b)"
    );
    let sql = fixture("../malformed-conflict.sql");
    let located = parse::parse_postgres_sql_with_function_sources(&sql);
    assert!(
        !super::super::recovery_failures(&located).is_empty(),
        "{located:?}"
    );
    assert!(parse::PreparedSql::new(&sql).parse_policy().is_err());
}
