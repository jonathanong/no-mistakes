use super::*;

fn fixture_options(path: &std::path::Path) -> serde_yaml::Value {
    let source = std::fs::read_to_string(path).unwrap();
    let config =
        crate::config::v2::discover::parse_v2_config_quiet(path.parent().unwrap(), &source, path)
            .unwrap();
    let rules = config.rule_applications("postgres-required-predicates");
    assert_eq!(rules.len(), 1, "{}", path.display());
    rules[0].options.clone()
}

fn nullable_fixture_profile() -> EmbeddedSqlOptions {
    let root = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors");
    let low = fixture_options(&root.join("low.yml"));
    let max = fixture_options(&root.join("max.yml"));
    assert_eq!(low, max);
    // Low effort skips this rule's embedded profile. max.yml has the same options.
    let path = root.join("max.yml");
    let source = std::fs::read_to_string(&path).unwrap();
    let config = crate::config::v2::discover::parse_v2_config_quiet(&root, &source, &path).unwrap();
    let mut profiles = crate::codebase::postgres::configured_embedded_sql_options(
        &config,
        &["postgres-required-predicates"],
    )
    .unwrap();
    assert_eq!(profiles.len(), 1, "{profiles:?}");
    let profile = profiles.pop().unwrap();
    assert_eq!(
        profile.import_specifier,
        max["importSpecifier"].as_str().unwrap()
    );
    let mut types = max["executorTypeNames"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    types.sort();
    assert_eq!(profile.executor_type_names, types);
    profile
}

fn nullable_profile_for(types: &[&str]) -> EmbeddedSqlOptions {
    let mut profile = nullable_fixture_profile();
    profile.executor_type_names = types.iter().map(|name| (*name).to_string()).collect();
    profile.executor_type_names.sort();
    profile
}

#[test]
fn nullable_custom_executor_types_keep_each_query_and_reassignment_controls() {
    let root = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors");
    let options = nullable_fixture_profile();
    for name in [
        "direct.ts",
        "subpath.ts",
        "null.ts",
        "pool.ts",
        "pool-subpath.ts",
        "top-direct.ts",
        "top-subpath.ts",
        "top-pool.ts",
        "top-pool-subpath.ts",
    ] {
        let path = root.join(name);
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = extract_embedded_sql_from_source(&path, &source, &options);
        assert_eq!(
            facts.calls.len(),
            if matches!(name, "direct.ts" | "pool.ts") {
                3
            } else {
                2
            },
            "{name}: {facts:?}"
        );
        assert!(facts.undeclared_executor_types.is_empty(), "{name}");
    }
    let path = root.join("controls.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert!(facts.calls.is_empty());
    assert_eq!(facts.undeclared_executor_types.len(), 5);
}

#[test]
fn known_literal_assignment_sources_cannot_be_overridden_by_type_casts() {
    let path = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors/literal-assignments.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = nullable_profile_for(&["PoolClient"]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert!(facts.calls.is_empty(), "{facts:?}");
    assert_eq!(facts.undeclared_executor_types.len(), 12);
}

#[test]
fn configured_callable_function_assignments_remain_executors() {
    let path = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors/callable.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = nullable_profile_for(&["Execute"]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    assert_eq!(facts.calls.len(), 4, "{facts:?}");
    assert!(facts.undeclared_executor_types.is_empty());
}

#[test]
fn nullable_optional_query_recovers_pool_client_and_pool_calls() {
    let path = crate::test_support::rule_fixture_root("postgres-required-predicates")
        .join("nullable-executors/optional-query.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = nullable_fixture_profile();
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    let mut sql = facts
        .calls
        .iter()
        .filter_map(|call| call.sql_text.clone())
        .collect::<Vec<_>>();
    sql.sort();
    assert_eq!(
        sql,
        [
            "UPDATE items SET a=1 WHERE id=$1".to_string(),
            "UPDATE items SET a=2 WHERE id=$1".to_string(),
            "UPDATE items SET a=3 WHERE id=$1 AND account_id=$2".to_string(),
            "UPDATE items SET a=4 WHERE id=$1 AND account_id=$2".to_string(),
        ],
        "{facts:?}"
    );
    assert!(facts.undeclared_executor_types.is_empty(), "{facts:?}");
}
