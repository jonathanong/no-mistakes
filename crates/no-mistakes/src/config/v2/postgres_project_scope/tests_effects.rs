use super::tests::parse;

#[test]
fn effects_project_synthesizes_selectors_once() {
    let yaml = r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
        factoryNames: [beginTransaction]
        typeNames: [TransactionQuery]
effects:
  postgres:
    project: db
    returnedMembers:
      - { module: "@example/db", export: writePool.connect, member: query }
rules:
  - rule: postgres-no-offset
    projects: [db]
"#;
    let mut config = parse(yaml).unwrap();
    assert!(config.effects["postgres"].functions.is_empty());
    assert!(config.effects["postgres"].targets.is_empty());
    assert_eq!(config.effects["postgres"].returned_executors.len(), 1);
    assert_eq!(
        config.effects["postgres"].returned_executors[0].export,
        "beginTransaction"
    );
    assert!(config.effects["postgres"].returned_executors[0]
        .category
        .is_none());
    assert_eq!(
        config.effects["postgres"].typed_executors[0].export,
        "TransactionQuery"
    );
    assert_eq!(
        config.effects["postgres"].returned_members[0].member,
        "query"
    );
    super::materialize(&mut config).unwrap();
    assert_eq!(config.effects["postgres"].returned_executors.len(), 1);
    assert_eq!(config.effects["postgres"].typed_executors.len(), 1);
    // `apply` itself is idempotent when the config-level flag is not what stops the call.
    let expanded = super::effects::apply(&mut config).unwrap();
    assert!(!expanded);
    assert_eq!(config.effects["postgres"].returned_executors.len(), 1);
    assert_eq!(config.effects["postgres"].typed_executors.len(), 1);
}

#[test]
fn effects_project_rejects_unknown_projects_and_restated_selectors() {
    let cases = [
        (
            r#"
effects:
  postgres:
    project: missing
"#,
            "not a known project",
        ),
        (
            r#"
projects:
  db:
    include: ["src/**"]
effects:
  postgres:
    project: db
"#,
            "has no postgres block",
        ),
        (
            r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
        factoryNames: [beginTransaction]
effects:
  postgres:
    project: db
    returnedExecutors:
      - { module: "@example/db", export: beginTransaction }
"#,
            "repeats",
        ),
        (
            r#"
projects:
  db:
    postgres:
      executors:
        factoryNames: [beginTransaction]
effects:
  postgres:
    project: db
"#,
            "importSpecifier",
        ),
        (
            r#"
effects:
  valkey:
    project: db
"#,
            "unsupported",
        ),
    ];
    for (yaml, needle) in cases {
        let error = parse(yaml).unwrap_err().to_string();
        assert!(error.contains(needle), "{error}");
    }
}

#[test]
fn rule_include_that_escapes_the_project_root_is_a_config_error() {
    let error = parse(
        r#"
projects:
  db:
    root: backend
    include: ["services/**"]
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    include: ["../outside/**"]
"#,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("escapes the project root"), "{error}");
}

#[test]
fn catalog_rules_inherit_only_the_catalog_path() {
    let config = parse(
        r#"
projects:
  db:
    postgres:
      schemaCatalogPath: catalog.json
      executors:
        importSpecifier: "@example/db"
        factoryNames: [beginTransaction]
rules:
  - rule: postgres-column-naming
    projects: [db]
  - rule: postgres-key-column-types
    projects: [db]
    options:
      allowedTypes: [uuid]
"#,
    )
    .unwrap();
    for rule in &config.rules {
        assert_eq!(
            rule.options["schemaCatalogPath"].as_str(),
            Some("catalog.json")
        );
        assert!(
            rule.options.get("importSpecifier").is_none(),
            "{}",
            rule.rule
        );
        assert!(rule.options.get("executorFactoryNames").is_none());
        assert!(rule.options.get("sqlIncludeLocked").is_none());
    }
    let naming: crate::codebase::rules::postgres_column_naming::Options =
        config.rules[0].try_rule_options().unwrap();
    assert_eq!(naming.schema_catalog_path, "catalog.json");
    let keys: crate::codebase::rules::postgres_key_column_types::Options =
        config.rules[1].try_rule_options().unwrap();
    assert_eq!(keys.schema_catalog_path, "catalog.json");
    assert_eq!(keys.allowed_types, ["uuid"]);
}
