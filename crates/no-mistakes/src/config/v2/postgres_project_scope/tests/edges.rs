use super::super::narrow::{prefix_pattern, root_prefix};
use super::parse;
use crate::config::v2::schema::NoMistakesConfig;

fn list(config: &NoMistakesConfig, index: usize, key: &str) -> Vec<String> {
    config.rules[index].options[key]
        .as_sequence()
        .unwrap_or_else(|| panic!("{key} should be a list"))
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn agreeing_projects_union_includes_and_keep_the_first_factory_order() {
    let config = parse(
        r#"
projects:
  a:
    include: ["src/a/**"]
    postgres:
      executors:
        importSpecifier: "@example/db"
        factoryNames: [write, read]
  b:
    include: ["src/b/**"]
    postgres:
      executors:
        importSpecifier: "@example/db"
        factoryNames: [read, write]
rules:
  - rule: postgres-no-offset
    projects: [a, b]
    include: ["src/b/queries/**"]
  - rule: postgres-no-offset
    projects: [a]
    include: ["src/a/**"]
"#,
    )
    .unwrap();
    assert_eq!(list(&config, 0, "executorFactoryNames"), ["write", "read"]);
    assert_eq!(config.rules[0].include, ["src/b/queries/**"]);
    assert_eq!(config.rules[1].include, ["src/a/**"]);
    assert_eq!(list(&config, 1, "executorFactoryNames"), ["write", "read"]);
}

#[test]
fn dot_root_does_not_prefix_and_empty_pattern_is_the_root() {
    let config = parse(
        r#"
projects:
  db:
    root: "."
    postgres:
      schemaCatalogPath: catalog.json
      sqlInclude: ["migrations/**"]
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
"#,
    )
    .unwrap();
    assert_eq!(config.rules[0].options["schemaCatalogPath"], "catalog.json");
    assert_eq!(list(&config, 0, "sqlInclude"), ["migrations/**"]);
    assert_eq!(root_prefix(Some(".")), None);
    assert_eq!(root_prefix(Some("/")), None);
    assert_eq!(prefix_pattern(Some("backend"), ""), "backend");
}

#[test]
fn repository_root_glob_and_dot_segments_stay_inside_the_project() {
    let rooted = parse(
        r#"
projects:
  db:
    include: ["/**"]
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    include: ["services/**"]
"#,
    )
    .unwrap();
    assert_eq!(rooted.rules[0].include, ["services/**"]);
    let dotted = parse(
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
    include: ["services/./api/**"]
"#,
    )
    .unwrap();
    assert_eq!(dotted.rules[0].include, ["services/./api/**"]);
}

#[test]
fn null_options_inherit_and_null_include_is_an_empty_list() {
    let null_options = parse(
        r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    options: null
"#,
    )
    .unwrap();
    assert_eq!(
        null_options.rules[0].options["importSpecifier"],
        "@example/db"
    );
    let null_include = parse(
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
    options:
      include: null
"#,
    )
    .unwrap();
    assert_eq!(
        null_include.rules[0].options["importSpecifier"],
        "@example/db"
    );
    assert!(null_include.rules[0].options["include"].is_null());
}

#[test]
fn malformed_options_unknown_projects_and_unproven_globs_are_errors() {
    let cases = [
        (
            r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db, missing]
"#,
            "missing",
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
    options: [1]
"#,
            "options must be a mapping",
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
    options:
      include: "services/**"
"#,
            "options.include must be a list of globs",
        ),
        (
            r#"
projects:
  db:
    postgres:
      sqlInclude: ["migrations/**"]
      executors:
        importSpecifier: "@example/db"
rules:
  - rule: postgres-no-offset
    projects: [db]
    options:
      sqlInclude: "migrations/**"
"#,
            "options.sqlInclude must be a list of globs",
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
    options:
      include: ["services/../other/**"]
"#,
            "options.include",
        ),
        (
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
    include: ["../../outside/**"]
"#,
            "escapes the project root",
        ),
    ];
    for (yaml, needle) in cases {
        let error = parse(yaml).unwrap_err().to_string();
        assert!(error.contains(needle), "{error}");
    }
}

#[test]
fn duplicate_factory_and_type_names_synthesize_one_selector_each() {
    let config = parse(
        r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
        factoryNames: [beginTransaction, beginTransaction]
        typeNames: [TransactionQuery, TransactionQuery]
effects:
  postgres:
    project: db
"#,
    )
    .unwrap();
    let returned: Vec<_> = config.effects["postgres"]
        .returned_executors
        .iter()
        .map(|selector| selector.export.as_str())
        .collect();
    let typed: Vec<_> = config.effects["postgres"]
        .typed_executors
        .iter()
        .map(|selector| selector.export.as_str())
        .collect();
    assert_eq!(returned, ["beginTransaction"]);
    assert_eq!(typed, ["TransactionQuery"]);
}

#[test]
fn the_same_name_in_factory_names_and_type_names_is_an_error() {
    let error = parse(
        r#"
projects:
  db:
    postgres:
      executors:
        importSpecifier: "@example/db"
        factoryNames: [beginTransaction]
        typeNames: [beginTransaction]
effects:
  postgres:
    project: db
"#,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("factoryNames"), "{error}");
    assert!(error.contains("typeNames"), "{error}");
    assert!(error.contains("beginTransaction"), "{error}");
}
