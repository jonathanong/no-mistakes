#[test]
fn quoted_table_arms_respect_temporary_identity_and_view_dependencies() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    // Prepared delimiter normalization keeps successive TABLE ASTs and names intact.
    assert!(!facts.parse_failed);
}

#[test]
fn quoted_table_arm_in_a_strictly_parsed_source_shadows_the_catalog() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-single.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
}

#[test]
fn repeated_quoted_table_arms_do_not_consume_tokens_twice() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-repeat.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    let statements: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.statement_start == Some((fact.line, fact.column)))
        .collect();
    assert_eq!(statements.len(), 3);
}

#[test]
fn skipped_create_table_arm_cannot_supply_a_later_query_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-skipped.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    let statements: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.statement_start == Some((fact.line, fact.column)))
        .collect();
    assert_eq!(statements.len(), 1);
}

#[test]
fn ignored_scalar_table_arm_cannot_supply_an_outer_arm_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-nested.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    let statements: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.statement_start == Some((fact.line, fact.column)))
        .collect();
    assert_eq!(statements.len(), 1);
}

#[test]
fn ignored_scalar_table_arm_does_not_steal_a_quoted_outer_identity() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-nested-reverse.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    let statements: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.statement_start == Some((fact.line, fact.column)))
        .collect();
    assert_eq!(statements.len(), 2);
    let right = &statements[0].query.items[1].kind;
    let crate::codebase::postgres::statements::SqlBoundItemKind::Query(right) = right else {
        panic!("the outer TABLE arm must be parsed");
    };
    assert!(matches!(
        &right.items[0].kind,
        crate::codebase::postgres::statements::SqlBoundItemKind::Opaque
    ));
    let after_drop = &statements[1].query.items[1].kind;
    let crate::codebase::postgres::statements::SqlBoundItemKind::Query(after_drop) = after_drop
    else {
        panic!("the replayed outer TABLE arm must be parsed");
    };
    assert!(matches!(
        &after_drop.items[0].kind,
        crate::codebase::postgres::statements::SqlBoundItemKind::Table(name)
            if name == "\"Accounts\""
    ));
}

#[test]
fn ignored_projection_table_arm_cannot_mask_a_derived_relation() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/temporary-table-quoted-projection.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let statements: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.statement_start == Some((fact.line, fact.column)))
        .collect();
    assert_eq!(statements.len(), 1);
}

#[test]
fn ignored_projection_table_arm_cannot_rename_a_direct_derived_relation() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/fixture/sql/table-quoted-derived.sql"
    ));
    let facts = crate::codebase::postgres::extract_sql_statement_facts(sql);
    assert!(!facts.parse_failed);
    let statements: Vec<_> = facts
        .bounds
        .iter()
        .filter(|fact| fact.statement_start == Some((fact.line, fact.column)))
        .collect();
    assert_eq!(statements.len(), 1);
}
