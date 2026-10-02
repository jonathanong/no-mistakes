use super::*;

#[test]
fn configured_schema_rules_borrow_one_prepared_migration_projection() {
    let path = unit("declaration-locations.sql");
    let root = path.parent().unwrap();
    let paths = [path.clone()];
    let mut config = config("maxBytes: 10\n");
    config.rules.push(RuleDef {
        rule: "postgres-no-add-column".into(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    });
    let sources = crate::codebase::rules::source_store_for_files(&paths);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        root,
        &paths,
        std::sync::Arc::clone(&sources),
        &config,
        &[RULE_ID, "postgres-no-add-column"],
    )
    .unwrap();
    let before = facts.postgres_schema_file(&path).unwrap();
    let findings =
        check_with_files_sources_and_facts(root, &config, &paths, &sources, &facts).unwrap();
    assert!(!findings.is_empty());
    assert!(
        crate::codebase::rules::postgres_no_add_column::check_with_files_sources_and_facts(
            root, &config, &paths, &sources, &facts
        )
        .unwrap()
        .is_empty()
    );
    assert!(std::ptr::eq(
        before,
        facts.postgres_schema_file(&path).unwrap()
    ));
    // Prepared consumers must not silently fall back to a second collection.
    assert!(check_with_files_sources_and_facts(
        root,
        &config,
        &paths,
        &sources,
        &crate::codebase::check_facts::CheckFactMap::default()
    )
    .unwrap_err()
    .to_string()
    .contains("prepared PostgreSQL facts are missing"));
}

fn declarations(name: &str) -> Vec<crate::codebase::postgres::SqlDeclaredIdentifier> {
    let sql = std::fs::read_to_string(unit(name)).unwrap();
    crate::codebase::postgres::extract_migration_facts(&sql).declared_identifiers
}

#[test]
fn token_locations_follow_modifiers_and_ignore_nested_comments_and_failed_statements() {
    let declarations = declarations("declaration-locations.sql");
    for (name, line) in [
        ("temporary_table", 10),
        ("unlogged_table", 11),
        ("after_invalid", 13),
        ("replaced_view", 14),
        ("explicit_column", 14),
        ("temporary_view", 15),
        ("replaced_trigger", 16),
        ("constraint_trigger", 18),
    ] {
        assert!(
            declarations
                .iter()
                .any(|item| item.name == name && item.line == line),
            "missing {name}:{line}: {declarations:?}"
        );
    }
    assert!(!declarations.iter().any(|item| item.name == "decoy_table"));
}

#[test]
fn outer_declaration_keeps_its_line_after_the_same_name_inside_a_routine() {
    let findings = run(&unit("declaration-locations.sql"), "maxBytes: 10\n");
    let repeated = findings
        .iter()
        .filter(|item| item.target.as_deref() == Some("repeated_table"))
        .collect::<Vec<_>>();
    assert_eq!(repeated.len(), 2, "{findings:?}");
    assert_eq!(repeated[0].line, 4);
    assert_eq!(repeated[1].line, 7);
}

#[test]
fn procedures_decode_unicode_comments_and_static_execute() {
    let declarations = declarations("procedure-tokens.sql");
    for (name, line) in [
        ("commented_procedure", 2),
        ("escaped_procedure", 3),
        ("custom_procedure", 4),
        ("dynamic_procedure", 7),
    ] {
        assert!(
            declarations
                .iter()
                .any(|item| item.name == name && item.line == line),
            "missing {name}:{line}: {declarations:?}"
        );
    }
}
