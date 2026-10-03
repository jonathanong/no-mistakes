use super::*;

fn scan(name: &str, files: &[&str]) -> Vec<RuleFinding> {
    let root = unit_fixture(name);
    let paths: Vec<_> = files.iter().map(|file| root.join(file)).collect();
    check_with_files(&root, &config_with_options(""), &paths).unwrap()
}

fn schema_and_writes(name: &str) -> Vec<RuleFinding> {
    scan(name, &["schema.sql", "writes.sql"])
}

#[test]
fn ctas_tables_keep_unknown_positional_order() {
    let findings = schema_and_writes("ctas-order");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn raise_exception_makes_rest_of_protected_block_unreachable() {
    let findings = schema_and_writes("raise-exception");
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("total"));
}

#[test]
fn quoted_and_unquoted_columns_stay_distinct_during_alter_replay() {
    let findings = schema_and_writes("quoted-column-alter");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn nested_declare_bindings_are_restored_after_the_block() {
    let findings = schema_and_writes("nested-shadow");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn create_if_not_exists_resolves_unqualified_relation_before_inserting() {
    let findings = schema_and_writes("create-if-not-exists");
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn quoted_control_keywords_do_not_open_conditional_scopes() {
    // The unconditional recreate replaces the generated table, so no finding remains.
    let findings = schema_and_writes("quoted-control-keyword");
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn migration_writes_use_the_catalog_as_of_that_statement() {
    let findings = scan(
        "migration-write-history",
        &["001_create.sql", "002_recreate.sql"],
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "001_create.sql");
    assert_eq!(findings[0].line, 2);
}

#[test]
fn same_line_ddl_precedes_a_following_write() {
    let findings = scan("same-line-migration", &["001_same_line.sql"]);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].line, 2);
}
