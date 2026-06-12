use super::comments::{strip_comments, strip_sql_comments};
use super::extract::{
    extract_path_regex_set, extract_set, extract_sql_enum, extract_ts_const_object_keys,
    extract_ts_const_object_property, extract_ts_string_union,
};
use super::object::matching_brace;
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
fn object_extraction_ignores_equals_in_type_annotations() {
    let source = r#"
const ROUTE_META: Record<string, () => { slug: string }> = {
  users: { slug: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_key_extraction_uses_only_top_level_keys() {
    let source = r#"
const ROUTE_META = {
  users: { slug: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_extraction_ignores_commented_matching_declarations() {
    let source = r#"
// const ROUTE_META = { legacy: { slug: "legacy" } };
const ROUTE_META = {
  users: { slug: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_extraction_skips_computed_keys() {
    let source = r#"
const ROUTE_META = {
  [ROUTES.users]: { slug: "users" },
  billing: { slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string()])
    );
}

#[test]
fn object_extraction_ignores_comment_braces_when_matching_body() {
    let source = r#"
const ROUTE_META = {
  // } old route map shape
  users: { slug: "users" },
  /* } block comment */
  billing: { slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn object_extraction_ignores_regex_literal_braces() {
    let source = r#"
const ROUTE_META = {
  users: { pattern: /}/, slug: "users" },
  billing: { pattern: /[{}]/, slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn object_extraction_handles_type_annotations_and_comments() {
    let source = r#"
export const ROUTE_META: Record<string, { slug: string }> = {
  // user route
  users: { slug: "users" },
  /* billing route */
  billing: { slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn object_extraction_handles_quoted_keys_unterminated_comments_and_final_values() {
    let source = r#"
const ROUTE_META = {
  "quoted-route": { slug: "quoted-route" },
  final: { slug: "final" }
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["final".to_string(), "quoted-route".to_string()])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["final".to_string(), "quoted-route".to_string()])
    );
    assert!(
        extract_ts_const_object_keys("const ROUTE_META = { invalidEntry }", "ROUTE_META")
            .is_empty()
    );
    assert!(extract_ts_const_object_keys(
        "const ROUTE_META = { /* intentionally unterminated\n}",
        "ROUTE_META"
    )
    .is_empty());
}

#[test]
fn object_property_extraction_requires_whole_key_match() {
    let source = r#"
const ROUTE_META = {
  users: { grid: "compact", id: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "id"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn object_property_extraction_ignores_nested_properties_and_spreads() {
    let source = r#"
const ROUTE_META = {
  ...COMMON_ROUTES,
  users: { seo: { slug: "seo-users" }, slug: "users" },
  billing: { slug: "billing" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["billing".to_string(), "users".to_string()])
    );
}

#[test]
fn object_property_extraction_ignores_terminal_spreads_and_non_literal_values() {
    assert!(
        extract_ts_const_object_keys("const ROUTE_META = { ...COMMON_ROUTES }", "ROUTE_META")
            .is_empty()
    );

    let source = r#"
const ROUTE_META = {
  users: "users",
  billing: { slug: ROUTE_BILLING },
};
"#;

    assert!(extract_ts_const_object_property(source, "ROUTE_META", "slug").is_empty());
}

#[test]
fn object_extraction_skips_spread_operands_with_nested_commas() {
    let source = r#"
const ROUTE_META = {
  ...buildRoutes("a", "b"),
  users: { slug: "users" },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["users".to_string()])
    );
}

#[test]
fn comment_strippers_preserve_quoted_comment_markers_and_newlines() {
    assert_eq!(
        strip_comments(r#"const value = "not // a comment"; // removed"#),
        r#"const value = "not // a comment"; __comment__"#
    );
    assert_eq!(
        strip_comments("const value = `not /* a comment */`; /* removed */\nnext"),
        "const value = `not /* a comment */`; __comment__\nnext"
    );
    assert_eq!(
        strip_sql_comments("SELECT 'it''s -- quoted'; -- removed\nSELECT 1"),
        "SELECT 'it''s -- quoted'; \nSELECT 1"
    );
    assert_eq!(
        strip_sql_comments("SELECT 1 /* removed\nstill removed */\nSELECT 2"),
        "SELECT 1 \n\nSELECT 2"
    );
}

#[test]
fn object_helpers_cover_comment_quotes_and_ignored_entries() {
    let source = r#"
const ROUTE_META = {
  // ignored: { slug: "ignored" },
  literal: { slug: "literal" },
  block: /* { } */ { slug: "block" },
  array: [",", { ignored: true }],
  ...buildRoutes({ nested: "value" }),
  templated: { slug: `template-${"ignored"}` },
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from([
            "array".to_string(),
            "block".to_string(),
            "literal".to_string(),
            "templated".to_string()
        ])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["literal".to_string()])
    );
}

#[test]
fn object_helpers_cover_ignored_tail_entries_and_quoted_literals() {
    let source = r#"
const ROUTE_META = {
  'quoted-key': { slug: 'quoted' },
  ...buildRoutes({ nested: "value" }),
  // line comment before ignored tail
  /* block comment before ignored tail */
  [dynamicKey()]
};
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["quoted-key".to_string()])
    );
    assert_eq!(
        extract_ts_const_object_property(source, "ROUTE_META", "slug"),
        BTreeSet::from(["quoted".to_string()])
    );
}

#[test]
fn object_helpers_stop_at_ignored_unclosed_tail_entries() {
    let source = r#"
const ROUTE_META = {
  literal: { slug: "literal" },
  ...buildRoutes({ nested: "value" })
};
const BROKEN = {
  literal: { slug: "literal" },
  /* unterminated
"#;

    assert_eq!(
        extract_ts_const_object_keys(source, "ROUTE_META"),
        BTreeSet::from(["literal".to_string()])
    );
    assert!(extract_ts_const_object_keys(source, "BROKEN").is_empty());
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
fn matching_brace_reports_unclosed_objects() {
    assert_eq!(matching_brace("const x = { a: 1", 10), None);
}

#[test]
fn matching_brace_handles_comments_strings_and_escapes() {
    assert_eq!(
        matching_brace(r#"const x = { value: "not } yet", done: true };"#, 10),
        Some(43)
    );
    assert_eq!(
        matching_brace(r#"const x = { value: 'escaped \' }', done: true };"#, 10),
        Some(46)
    );
    assert_eq!(
        matching_brace("const x = { value: `template }`, done: true };", 10),
        Some(44)
    );
    assert_eq!(
        matching_brace("const x = { // } comment\n done: true };", 10),
        Some(37)
    );
    assert_eq!(
        matching_brace("const x = { /* } comment */ done: true };", 10),
        Some(39)
    );
    assert_eq!(matching_brace("const x = { value: a / b };", 10), Some(25));
    assert_eq!(matching_brace("const x = { /* unterminated", 10), None);
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
