use super::extract::{
    extract_markdown_table_code_cells, extract_sql_enum, extract_ts_array_literal,
    extract_ts_const_array_property, extract_yaml_sequence, ExtractedSet,
};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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
fn extracts_yaml_ts_array_and_markdown_sets() {
    let root = fixture_root("fixture");
    let types = std::fs::read_to_string(root.join("src/types.ts")).unwrap();
    let workspace = std::fs::read_to_string(root.join("pnpm-workspace.yaml")).unwrap();
    let docs = std::fs::read_to_string(root.join("docs/dependency-updates.md")).unwrap();

    assert_eq!(
        extract_yaml_sequence(&workspace, "minimumReleaseAgeExclude"),
        BTreeSet::from([
            "@acme/api".to_string(),
            "@acme/cli".to_string(),
            "@acme/web".to_string()
        ])
    );
    assert_eq!(
        extract_ts_const_array_property(&types, "FIRST_PARTY_EXEMPTIONS", "name"),
        BTreeSet::from([
            "@acme/api".to_string(),
            "@acme/docs".to_string(),
            "@acme/web".to_string()
        ])
    );
    assert_eq!(
        extract_ts_array_literal(&types, "FIRST_PARTY_NAMES"),
        BTreeSet::from(["@acme/api".to_string(), "@acme/web".to_string()])
    );
    assert_eq!(
        extract_markdown_table_code_cells(&docs),
        BTreeSet::from(["@acme/api".to_string(), "@acme/web".to_string()])
    );
}

#[test]
fn compares_yaml_ts_glob_and_markdown_sets() {
    let root = fixture_root("fixture");
    let files = vec![
        root.join("pnpm-workspace.yaml"),
        root.join("src/types.ts"),
        root.join(".github/dependabot.yml"),
        root.join("docs/dependency-updates.md"),
    ];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sets:
  - name: workspaceExcludes
    file: pnpm-workspace.yaml
    kind: yaml-sequence
    key: minimumReleaseAgeExclude
  - name: registry
    file: src/types.ts
    kind: ts-const-array-property
    target: FIRST_PARTY_EXEMPTIONS
    property: name
  - name: dependabotGlobs
    file: .github/dependabot.yml
    kind: yaml-sequence
    key: updates.0.cooldown.exclude
  - name: docsMentions
    file: docs/dependency-updates.md
    kind: markdown-table-code-cells
comparisons:
  - left: workspaceExcludes
    right: registry
  - left: registry
    right: dependabotGlobs
    mode: glob-coverage
  - left: registry
    right: docsMentions
    mode: mention
"#,
        ),
        &files,
    )
    .unwrap();
    let body = format!("{findings:?}");

    assert_eq!(findings.len(), 4, "{body}");
    assert!(
        body.contains("workspaceExcludes contains `@acme/cli`"),
        "{body}"
    );
    assert!(body.contains("registry contains `@acme/docs`"), "{body}");
    assert!(
        body.contains("no glob in dependabotGlobs covers it"),
        "{body}"
    );
    assert!(body.contains("docsMentions does not mention it"), "{body}");
}

#[test]
fn glob_coverage_reports_invalid_globs() {
    let root = fixture_root("fixture");
    let files = vec![root.join("src/types.ts"), root.join("pnpm-workspace.yaml")];
    let findings = check_with_files(
        &root,
        &config(
            r#"
sets:
  - name: names
    file: src/types.ts
    kind: ts-array-literal
    target: FIRST_PARTY_NAMES
  - name: globs
    file: pnpm-workspace.yaml
    kind: yaml-sequence
    key: invalidGlobs
comparisons:
  - left: names
    right: globs
    mode: glob-coverage
"#,
        ),
        &files,
    )
    .unwrap();

    assert!(
        findings
            .iter()
            .any(|finding| finding.message.contains("invalid glob")),
        "unexpected findings: {findings:?}"
    );
}

#[test]
fn config_set_extractors_cover_edge_cases() {
    assert!(extract_yaml_sequence(":", "packages").is_empty());
    assert!(extract_yaml_sequence("packages: nope", "packages").is_empty());
    assert!(extract_yaml_sequence("updates: []", "updates.0.exclude").is_empty());
    assert_eq!(
        extract_yaml_sequence("packages:\n  - one\n  - 2\n", "packages"),
        BTreeSet::from(["one".to_string()])
    );

    assert!(extract_ts_array_literal("const OTHER = ['a'];", "NAMES").is_empty());
    assert_eq!(
        extract_ts_array_literal(
            r#"const NAMES = ["a\"b", `template`, notString, 'single'];"#,
            "NAMES"
        ),
        BTreeSet::from([
            "a\"b".to_string(),
            "single".to_string(),
            "template".to_string()
        ])
    );
    assert!(extract_ts_array_literal(
        r#"const NAMES = getNames(); const OTHER = ["api"];"#,
        "NAMES"
    )
    .is_empty());
    assert_eq!(
        extract_ts_array_literal(
            r#"const NAMES = [
  // keep pinned
  "@acme/api",
  /*
   * keep local
   */
  "@acme/web",
];"#,
            "NAMES"
        ),
        BTreeSet::from(["@acme/api".to_string(), "@acme/web".to_string()])
    );
    assert!(extract_ts_array_literal(r#"const NAMES = ["unterminated];"#, "NAMES").is_empty());
    assert!(
        extract_ts_const_array_property("const OTHER = [{ name: 'api' }];", "ITEMS", "name")
            .is_empty()
    );
    assert!(extract_ts_const_array_property(
        r#"const ITEMS = [{ name: "unterminated }];"#,
        "ITEMS",
        "name"
    )
    .is_empty());
    assert!(extract_ts_const_array_property(
        r#"const ITEMS = getItems(); const OTHER = [{ name: "api" }];"#,
        "ITEMS",
        "name"
    )
    .is_empty());
    assert_eq!(
        extract_ts_const_array_property(
            r#"const ITEMS = [
  // first-party package
  { name: "api" },
  "ignored",
  /* documented in policy table */
  { name: `web`, other: "x" },
];"#,
            "ITEMS",
            "name"
        ),
        BTreeSet::from(["api".to_string(), "web".to_string()])
    );
    assert!(extract_yaml_sequence("packages:\n  nested: []\n", "packages.0").is_empty());
    assert!(extract_sql_enum("CREATE TYPE status AS ENUM ('open'", "status").is_empty());
}

#[test]
fn comparison_modes_cover_defaults_custom_messages_and_unknown_modes() {
    let left = ExtractedSet {
        file: "left.ts".to_string(),
        values: BTreeSet::from(["api".to_string(), "web".to_string()]),
    };
    let right = ExtractedSet {
        file: "right.md".to_string(),
        values: BTreeSet::from(["api".to_string()]),
    };

    let mut findings = Vec::new();
    super::comparison::compare(
        &left,
        &right,
        &Comparison {
            left: "left".to_string(),
            right: "right".to_string(),
            message: Some("sets differ".to_string()),
            ..Default::default()
        },
        &mut findings,
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].message, "sets differ");

    findings.clear();
    super::comparison::compare(
        &left,
        &right,
        &Comparison {
            left: "left".to_string(),
            right: "right".to_string(),
            mode: "unknown".to_string(),
            ..Default::default()
        },
        &mut findings,
    );
    assert!(findings.is_empty());
}
