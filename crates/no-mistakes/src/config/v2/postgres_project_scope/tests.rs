use std::path::Path;

use crate::config::v2::discover::parse_v2_config_quiet;
use crate::config::v2::schema::NoMistakesConfig;

fn parse(yaml: &str) -> anyhow::Result<NoMistakesConfig> {
    parse_v2_config_quiet(yaml, Path::new(".no-mistakes.yml"))
}

fn strings(config: &NoMistakesConfig, index: usize, key: &str) -> Vec<String> {
    config.rules[index].options[key]
        .as_sequence()
        .unwrap_or_else(|| panic!("{key} should be a list"))
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

const SHARED: &str = r#"
projects:
  db:
    root: backend
    include: ["services/**"]
    postgres:
      sqlInclude: ["migrations/**"]
      schemaCatalogPath: data-stores/psql/schema-snapshot/catalog.json
      unanalyzableSql: ignore
      executors:
        importSpecifier: "@example/db"
        factoryNames: [beginTransaction]
        typeNames: [TransactionQuery]
        trustedSqlTags: [{ module: "@example/db", name: sql }]
rules:
  - rule: postgres-no-offset
    projects: [db]
  - rule: postgres-require-query-annotation
    projects: [db]
"#;

#[test]
fn shared_project_injects_equal_executor_options_and_one_profile() {
    let config = parse(SHARED).unwrap();
    assert!(config.postgres_project_scope_materialized);
    for rule in &config.rules {
        assert_eq!(rule.options["importSpecifier"], "@example/db");
        assert_eq!(
            strings(&config, 0, "executorFactoryNames"),
            strings(&config, 1, "executorFactoryNames")
        );
        assert_eq!(
            rule.options["schemaCatalogPath"],
            "backend/data-stores/psql/schema-snapshot/catalog.json"
        );
        assert_eq!(rule.options["sqlIncludeLocked"], true);
        assert!(rule.options.get("executorNames").is_none());
    }
    assert_eq!(
        strings(&config, 0, "sqlInclude"),
        ["backend/migrations/**".to_string()]
    );
    let profiles = crate::codebase::postgres::configured_embedded_sql_options(
        &config,
        &["postgres-no-offset", "postgres-require-query-annotation"],
    )
    .unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].import_specifier, "@example/db");
    assert_eq!(
        profiles[0].executor_names,
        ["query".to_string(), "read".to_string(), "write".to_string()]
    );
    assert_eq!(profiles[0].executor_factory_names, ["beginTransaction"]);
    assert_eq!(profiles[0].executor_type_names, ["TransactionQuery"]);
    assert!(profiles[0]
        .executor_factory_names
        .iter()
        .all(|name| name != "query"));
}

#[test]
fn rule_cannot_widen_or_restate_postgres_project_scope() {
    let cases = [
        (
            r#"
projects:
  db:
    postgres:
      sqlInclude: ["migrations/**/*.sql"]
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    scope: repository
"#,
            "wider pass",
        ),
        (
            r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    options:
      importSpecifier: "@example/db"
"#,
            "restates",
        ),
        (
            r#"
projects:
  db:
    postgres:
      sqlInclude: ["migrations/**/*.sql"]
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    options:
      sqlInclude: ["**/*.sql"]
"#,
            "not a proven narrower glob",
        ),
        (
            r#"
projects:
  db:
    include: ["services/**"]
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    include: ["lib/**"]
"#,
            "not a proven narrower glob",
        ),
        (
            r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    options:
      sqlIncludeLocked: true
"#,
            "sqlIncludeLocked",
        ),
    ];
    for (yaml, needle) in cases {
        let error = parse(yaml).unwrap_err().to_string();
        assert!(error.contains(needle), "{error}");
    }
}

#[test]
fn proven_narrow_pattern_is_kept_and_unproven_parent_is_refused() {
    let kept = parse(
        r#"
projects:
  db:
    root: backend
    include: ["services/**"]
    postgres:
      sqlInclude: ["migrations/**"]
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    include: ["services/api/**"]
    options:
      include: ["backend/services/api/**"]
      sqlInclude: ["backend/migrations/v1/**/*.sql"]
      exclude: ["**/*.test.*"]
"#,
    )
    .unwrap();
    assert_eq!(kept.rules[0].include, ["services/api/**"]);
    assert_eq!(
        strings(&kept, 0, "sqlInclude"),
        ["backend/migrations/v1/**/*.sql".to_string()]
    );
    let unproven = parse(
        r#"
projects:
  db:
    postgres:
      sqlInclude: ["migrations/**/*.sql"]
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    options:
      sqlInclude: ["migrations/v1/**/*.sql"]
"#,
    )
    .unwrap_err()
    .to_string();
    assert!(
        unproven.contains("not a proven narrower glob"),
        "{unproven}"
    );
}

#[test]
fn empty_project_sql_include_rejects_a_rule_list_and_locks_an_empty_one() {
    let widened = parse(
        r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-identifier-length
    projects: [db]
    options:
      sqlInclude: ["**/*.sql"]
"#,
    )
    .unwrap_err()
    .to_string();
    assert!(
        widened.contains("widens an empty project sqlInclude"),
        "{widened}"
    );
    let locked = parse(
        r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-identifier-length
    projects: [db]
"#,
    )
    .unwrap();
    assert!(strings(&locked, 0, "sqlInclude").is_empty());
    assert_eq!(locked.rules[0].options["sqlIncludeLocked"], true);
    let opts: crate::codebase::rules::postgres_identifier_length::Options =
        locked.rules[0].try_rule_options().unwrap();
    assert!(opts.sql_include.is_empty());
    assert!(opts.sql_include_locked);
}

#[test]
fn per_rule_executors_stay_valid_without_a_postgres_project() {
    let config = parse(
        r#"
rules:
  - rule: postgres-no-offset
    scope: repository
    options:
      importSpecifier: "@example/db"
"#,
    )
    .unwrap();
    assert!(!config.postgres_project_scope_materialized);
    assert!(config.rules[0].options.get("sqlIncludeLocked").is_none());
    assert_eq!(config.rules[0].options["importSpecifier"], "@example/db");
}

#[test]
fn empty_import_specifier_does_not_select_every_query() {
    let error = parse(
        r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "  "
        factoryNames: []
rules:
  - rule: postgres-no-offset
    projects: [db]
"#,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("importSpecifier"), "{error}");
    let naming = parse(
        r#"
projects:
  db:
    root: backend
    postgres:
      schemaCatalogPath: catalog.json
      executors:
        importSpecifier: "@example/db"
        factoryNames: [beginTransaction]
        typeNames: [TransactionQuery]
        trustedSqlTags: [{ module: "@example/db", name: sql }]
rules:
  - rule: postgres-column-naming
    projects: [db]
  - rule: postgres-key-column-types
    projects: [db]
"#,
    )
    .unwrap();
    let column: crate::codebase::rules::postgres_column_naming::Options =
        naming.rules[0].try_rule_options().unwrap();
    assert_eq!(column.schema_catalog_path, "backend/catalog.json");
    assert_eq!(column.import_specifier, "@example/db");
    assert!(column.sql_include_locked);
    let keys: crate::codebase::rules::postgres_key_column_types::Options =
        naming.rules[1].try_rule_options().unwrap();
    assert_eq!(keys.executor_factory_names, ["beginTransaction"]);
    assert!(keys.sql_include_locked);
}

#[test]
fn disagreeing_projects_and_mixed_references_are_config_errors() {
    let disagree = parse(
        r#"
projects:
  a:
    postgres:
      executors:
        importSpecifier: "@example/a"
  b:
    postgres:
      executors:
        importSpecifier: "@example/b"
rules:
  - rule: postgres-no-offset
    projects: [a, b]
"#,
    )
    .unwrap_err()
    .to_string();
    assert!(
        disagree.contains("disagree on importSpecifier"),
        "{disagree}"
    );
    let mixed = parse(
        r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
  api:
    include: ["src/**"]
rules:
  - rule: postgres-no-offset
    projects: [db, api]
"#,
    )
    .unwrap_err()
    .to_string();
    assert!(mixed.contains("without a postgres block"), "{mixed}");
}

#[test]
fn disabled_rules_and_plain_projects_keep_today_behavior() {
    let disabled = parse(
        r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    enabled: false
    projects: [db]
    scope: repository
    options:
      importSpecifier: "@other/db"
"#,
    )
    .unwrap();
    assert!(!disabled.postgres_project_scope_materialized);
    assert_eq!(disabled.rules[0].options["importSpecifier"], "@other/db");
    let plain = parse(
        r#"
projects:
  api:
    include: ["src/**"]
rules:
  - rule: postgres-no-offset
    projects: [api]
    options:
      importSpecifier: "@example/db"
"#,
    )
    .unwrap();
    assert!(plain.rules[0].options.get("sqlIncludeLocked").is_none());
}

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
