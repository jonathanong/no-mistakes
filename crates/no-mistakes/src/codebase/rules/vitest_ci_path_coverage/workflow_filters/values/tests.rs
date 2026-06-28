use super::*;

fn fixture_root(name: &str) -> std::path::PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/vitest-ci-path-coverage")
            .join(name),
    )
}

#[test]
fn parse_filters_value_reports_invalid_inline_and_missing_external_files() {
    let root = fixture_root("fixture");
    let mut findings = Vec::new();

    assert!(parse_filters_value(
        &root,
        ".github/workflows/ci.yml",
        "job",
        "filter",
        "[",
        &mut findings
    )
    .is_none());
    assert!(parse_filters_value(
        &root,
        ".github/workflows/ci.yml",
        "job",
        "filter",
        ".github/missing.yml",
        &mut findings
    )
    .is_none());

    assert_eq!(findings.len(), 2);
    assert!(findings[0].message.contains("is not valid YAML"));
    assert!(findings[1].message.contains("could not be read"));
}

#[test]
fn parse_filters_value_loads_external_files_and_reports_invalid_external_yaml() {
    let root = fixture_root("malformed");
    let mut findings = Vec::new();

    let value = parse_filters_value(
        &root,
        ".github/workflows/ci.yml",
        "job",
        "filter",
        ".github/valid-filters.yml",
        &mut findings,
    )
    .unwrap();
    assert!(value.as_mapping().unwrap().contains_key("backend"));

    assert!(parse_filters_value(
        &root,
        ".github/workflows/ci.yml",
        "job",
        "filter",
        ".github/invalid-filters.yml",
        &mut findings
    )
    .is_none());

    assert_eq!(findings.len(), 1);
    assert!(findings[0]
        .message
        .contains("file `.github/invalid-filters.yml` is not valid YAML"));
}

#[test]
fn filter_predicates_recurses_through_supported_shapes() {
    let value: Value = serde_yaml::from_str(
        r#"
- "src/**/*.ts"
- added|modified:
    - "pkg/**/*.ts"
- added:
    nested: "api/**/*.ts"
- deleted:
    - "deleted/**/*.ts"
- paths:
    - "web/**/*.tsx"
- other:
    nested: "scripts/**/*.mts"
- 1
"#,
    )
    .unwrap();

    assert_eq!(
        filter_predicates(&value),
        vec![
            vec!["src/**/*.ts".to_string()],
            vec!["pkg/**/*.ts".to_string()],
        ]
    );
}

#[test]
fn change_type_list_values_stay_single_predicate_alternatives() {
    let value: Value = serde_yaml::from_str(
        r#"
added|modified:
  - "src/**"
  - "lib/**"
"#,
    )
    .unwrap();

    assert_eq!(
        filter_predicates(&value),
        vec![vec!["src/**".to_string(), "lib/**".to_string()]]
    );
}
