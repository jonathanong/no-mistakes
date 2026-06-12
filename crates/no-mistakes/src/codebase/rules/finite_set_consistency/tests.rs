use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/finite-set-consistency")
            .join(name),
    )
}

fn config(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn compares_extracted_string_sets() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("src/routes/admin.ts"),
        root.join("src/routes/billing.ts"),
        root.join("src/routes/users.ts"),
        root.join("src/types.ts"),
        root.join("schema.sql"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sets:
  - name: routeType
    file: src/types.ts
    kind: ts-string-union
    target: RouteName
  - name: routeFiles
    kind: path-regex-capture
    pattern: '^src/routes/(?<value>[^/]+)\.ts$'
  - name: routeEnum
    file: schema.sql
    kind: sql-enum
    target: route_name
comparisons:
  - left: routeType
    right: routeFiles
  - left: routeType
    right: routeEnum
"#,
        ),
        &files,
    )
    .unwrap();

    assert_eq!(findings.len(), 3);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("routeFiles contains `admin`")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("routeType contains `settings`")));
}

#[test]
fn extracts_const_object_keys_and_property_values() {
    let root = fixture_root("fixture");
    let source = std::fs::read_to_string(root.join("src/types.ts")).unwrap();

    assert!(extract_ts_const_object_keys(&source, "ROUTE_META").contains("users"));
    assert!(extract_ts_const_object_property(&source, "ROUTE_META", "slug").contains("billing"));
}

#[test]
fn ignores_incomplete_set_specs_and_comparisons() {
    let root = fixture_root("fixture");
    let files = vec![root.join("src/types.ts")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sets:
  - name: ""
    file: src/types.ts
    kind: ts-string-union
    target: RouteName
  - name: known
    file: src/types.ts
    kind: unknown-kind
comparisons:
  - left: known
    right: missing
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn extraction_helpers_return_empty_sets_when_targets_are_missing() {
    assert!(extract_ts_string_union("type Other = 'a';", "Missing").is_empty());
    assert!(extract_ts_const_object_keys("const Other = { a: 1 };", "Missing").is_empty());
    assert!(extract_ts_const_object_property(
        "const Other = { a: { slug: 'a' } };",
        "Missing",
        "slug"
    )
    .is_empty());
    assert!(extract_sql_enum("CREATE TYPE other AS ENUM ('a')", "missing").is_empty());
}

#[test]
fn object_property_extraction_handles_nested_quoted_braces() {
    let source = r#"
const ROUTE_META = {
  users: { label: "brace \" }", slug: "users" },
  billing: { slug: 'billing' },
};
"#;

    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn matching_brace_reports_unclosed_objects() {
    assert_eq!(matching_brace("const x = { a: 1", 10), None);
}

#[test]
fn path_regex_set_uses_spec_file_when_present() {
    let root = fixture_root("fixture");
    let files = vec![root.join("src/routes/users.ts")];
    let spec = SetSpec {
        name: "routes".to_string(),
        file: "routes-index".to_string(),
        kind: "path-regex-capture".to_string(),
        pattern: r#"^src/routes/([^/]+)\.ts$"#.to_string(),
        ..Default::default()
    };
    let extracted = extract_path_regex_set(&root, &spec, &files).unwrap();

    assert_eq!(extracted.file, "routes-index");
    assert_eq!(extracted.values, BTreeSet::from(["users".to_string()]));
}

#[test]
fn extract_set_supports_const_object_property_specs() {
    let root = fixture_root("fixture");
    let spec = SetSpec {
        file: "src/types.ts".to_string(),
        kind: "ts-const-object-property".to_string(),
        target: "ROUTE_META".to_string(),
        property: "slug".to_string(),
        ..Default::default()
    };
    let extracted = extract_set(&root, &spec, &[]).unwrap();

    assert!(extracted.values.contains("users"));
}
