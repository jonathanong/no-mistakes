use super::extract::{
    extract_path_regex_set, extract_set, extract_sql_enum, extract_ts_const_object_keys,
    extract_ts_const_object_property, extract_ts_string_union,
};
use super::*;
use crate::config::v2::{
    schema::{Project, RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::BTreeSet;
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
fn project_scoped_set_files_resolve_against_project_root() {
    let root = fixture_root("project");
    let files = vec![
        root.join("packages/app/src/types.ts"),
        root.join("packages/app/src/routes/users.ts"),
    ];
    let mut config = config(
        r#"
sets:
  - name: routeType
    file: src/types.ts
    kind: ts-string-union
    target: RouteName
  - name: routeFiles
    kind: path-regex-capture
    pattern: '^packages/app/src/routes/(?<value>[^/]+)\.ts$'
comparisons:
  - left: routeType
    right: routeFiles
"#,
    );
    config.projects.insert(
        "app".to_string(),
        Project {
            root: Some("packages/app".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["app".to_string()];

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn project_scoped_set_files_read_each_project_root() {
    let root = fixture_root("project");
    let files = vec![
        root.join("packages/app/src/types.ts"),
        root.join("packages/app/src/routes/users.ts"),
        root.join("packages/admin/src/types.ts"),
        root.join("packages/admin/src/routes/admin.ts"),
    ];
    let mut config = config(
        r#"
sets:
  - name: routeType
    file: src/types.ts
    kind: ts-string-union
    target: RouteName
  - name: routeFiles
    kind: path-regex-capture
    pattern: '^src/routes/(?<value>[^/]+)\.ts$'
comparisons:
  - left: routeType
    right: routeFiles
"#,
    );
    config.projects.insert(
        "app".to_string(),
        Project {
            root: Some("packages/app".to_string()),
            ..Default::default()
        },
    );
    config.projects.insert(
        "admin".to_string(),
        Project {
            root: Some("packages/admin".to_string()),
            ..Default::default()
        },
    );
    config.rules[0].scope = None;
    config.rules[0].projects = vec!["app".to_string(), "admin".to_string()];

    let findings = check_with_files(&root, &config, &files).unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
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
    assert!(extract_ts_const_object_keys(
        "const ROUTE_META: Record<string, string>;",
        "ROUTE_META"
    )
    .is_empty());
}

#[test]
fn extraction_helpers_parse_blank_lines_and_escaped_quotes() {
    assert_eq!(
        extract_ts_string_union(
            r#"type RouteName =
  | "users"

  | "billing"
  | "a\"b";
"#,
            "RouteName"
        ),
        BTreeSet::from([
            "a\"b".to_string(),
            "billing".to_string(),
            "users".to_string()
        ])
    );
    assert_eq!(
        extract_sql_enum(
            "CREATE TYPE route_name AS ENUM ('can''t', 'users');",
            "route_name"
        ),
        BTreeSet::from(["can't".to_string(), "users".to_string()])
    );
}

#[test]
fn string_union_extraction_ignores_semicolons_inside_literals() {
    assert_eq!(
        extract_ts_string_union(r#"type RouteName = "a;b" | "c";"#, "RouteName"),
        BTreeSet::from(["a;b".to_string(), "c".to_string()])
    );
    assert_eq!(
        extract_ts_string_union(
            r#"type RouteName = 'single;quoted' | `template\`;still quoted` | "done";
const ignored = "ignored";
"#,
            "RouteName",
        ),
        BTreeSet::from([
            "done".to_string(),
            "single;quoted".to_string(),
            "template`;still quoted".to_string()
        ])
    );
}

#[test]
fn extraction_helpers_stop_semicolonless_unions_before_declare() {
    assert_eq!(
        extract_ts_string_union(
            r#"type RouteName =
  | "users"
declare const ignored: "ignored"
"#,
            "RouteName"
        ),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn sql_enum_extraction_ignores_commented_matching_definitions() {
    let source = r#"
-- CREATE TYPE route_name AS ENUM ('legacy');
CREATE TYPE route_name AS ENUM ('users', 'billing');
"#;

    assert_eq!(
        extract_sql_enum(source, "route_name"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn sql_enum_extraction_ignores_block_commented_matching_definitions() {
    let source = r#"
/* CREATE TYPE route_name AS ENUM ('legacy'); */
CREATE TYPE route_name AS ENUM ('users', 'billing');
"#;

    assert_eq!(
        extract_sql_enum(source, "route_name"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn sql_enum_extraction_preserves_comment_token_boundaries() {
    assert_eq!(
        extract_sql_enum(
            "CREATE/* comment */TYPE route_name AS ENUM ('users')",
            "route_name"
        ),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn string_union_extraction_does_not_require_semicolons() {
    assert_eq!(
        extract_ts_string_union("type RouteName = 'users' | 'billing'\n", "RouteName"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn string_union_extraction_stops_before_following_declarations() {
    let source = r#"
type RouteName = "users" | "billing"
export const LABELS = { users: "Users" };
"#;

    assert_eq!(
        extract_ts_string_union(source, "RouteName"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn string_union_extraction_handles_crlf_before_following_declaration() {
    let source = std::fs::read_to_string(fixture_root("fixture").join("src/crlf-union.ts"))
        .unwrap()
        .replace('\n', "\r\n");

    assert_eq!(
        extract_ts_string_union(&source, "RouteName"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn string_union_extraction_stops_before_blank_lines_and_declaration_keywords() {
    assert_eq!(
        extract_ts_string_union(
            "type RouteName = 'users'\n\nconst x = 'ignored'",
            "RouteName"
        ),
        BTreeSet::from(["users".to_string()])
    );

    for keyword in [
        "import",
        "const",
        "let",
        "var",
        "type",
        "interface",
        "class",
        "enum",
        "function",
    ] {
        let source = format!("type RouteName = 'users'\n{keyword} Next = 'ignored'");
        assert_eq!(
            extract_ts_string_union(&source, "RouteName"),
            BTreeSet::from(["users".to_string()]),
            "keyword {keyword} should terminate the union"
        );
    }
}

#[test]
fn string_union_extraction_ignores_commented_literals() {
    let source = r#"
type RouteName =
  | "users"
  // | "legacy"
  | "billing" /* "example" */
"#;

    assert_eq!(
        extract_ts_string_union(source, "RouteName"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn string_union_extraction_ignores_commented_matching_aliases() {
    let source = r#"
// type RouteName = "legacy"
type RouteName = "users" | "billing"
"#;

    assert_eq!(
        extract_ts_string_union(source, "RouteName"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn string_union_extraction_ignores_matching_alias_text_inside_literals() {
    let source = r#"
const docs = 'type RouteName = "legacy"';
const escapedDocs = "type RouteName = \"escaped\"";
const templateDocs = `type RouteName = "template"`;
type RouteName = "users" | "billing";
"#;

    assert_eq!(
        extract_ts_string_union(source, "RouteName"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn string_union_extraction_supports_ts_single_quote_escapes() {
    assert_eq!(
        extract_ts_string_union(r#"type RouteName = 'can\'t' | 'users';"#, "RouteName"),
        BTreeSet::from(["can't".to_string(), "users".to_string()])
    );
}

#[test]
fn string_union_extraction_supports_template_literal_members() {
    assert_eq!(
        extract_ts_string_union(r#"type RouteName = `users` | `billing`;"#, "RouteName"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn sql_enum_extraction_ignores_semicolons_inside_labels() {
    assert_eq!(
        extract_sql_enum(
            "CREATE TYPE route_name AS ENUM ('a;b', 'can''t');",
            "route_name"
        ),
        BTreeSet::from(["a;b".to_string(), "can't".to_string()])
    );
}

#[test]
fn object_extraction_preserves_repository_relative_error_files() {
    let root = fixture_root("fixture");
    let spec = SetSpec {
        name: "missing".to_string(),
        file: "src/missing.ts".to_string(),
        kind: "ts-string-union".to_string(),
        target: "RouteName".to_string(),
        ..Default::default()
    };

    assert!(extract_set(&root, &spec, &[], &[]).is_err());
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
    let extracted = extract_path_regex_set(&root, &spec, &files, &[]).unwrap();

    assert_eq!(extracted.file, "routes-index");
    assert_eq!(extracted.values, BTreeSet::from(["users".to_string()]));
}

#[test]
fn path_regex_set_matches_project_relative_paths() {
    let root = fixture_root("fixture");
    let project_root = root.join("packages/app");
    let files = vec![project_root.join("src/routes/users.ts")];
    let spec = SetSpec {
        name: "routes".to_string(),
        file: "routes-index".to_string(),
        kind: "path-regex-capture".to_string(),
        pattern: r#"^src/routes/([^/]+)\.ts$"#.to_string(),
        ..Default::default()
    };
    let extracted = extract_path_regex_set(&root, &spec, &files, &[project_root]).unwrap();

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
    let extracted = extract_set(&root, &spec, &[], &[]).unwrap();

    assert!(extracted.values.contains("users"));
}
