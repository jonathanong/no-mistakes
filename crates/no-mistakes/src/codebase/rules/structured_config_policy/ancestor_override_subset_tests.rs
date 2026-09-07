use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::Path;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/structured-config-policy/ancestor-override-subset"),
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

fn inventory(root: &Path, rels: &[&str]) -> Vec<PathBuf> {
    rels.iter().map(|rel| root.join(rel)).collect()
}

fn oxlintrc_yaml() -> &'static str {
    r#"
policies:
  - files: ["**/.oxlintrc.json"]
    when:
      - key: extends
    valueAssertions:
      - kind: ancestor-override-subset
"#
}

fn oxlintrc_inventory(root: &Path) -> Vec<PathBuf> {
    inventory(
        root,
        &[
            ".oxlintrc.json",
            "extra.jsonc",
            "invalid.json",
            "cycle/peer.json",
            "nested/.oxlintrc.json",
            "nested/file.ts",
            "lost/.oxlintrc.json",
            "lost/file.ts",
            "drift/.oxlintrc.json",
            "drift/file.ts",
            "superset/.oxlintrc.json",
            "superset/file.ts",
            "still-matches/.oxlintrc.json",
            "still-matches/file.ts",
            "unmatched/.oxlintrc.json",
            "unmatched/file.ts",
            "standalone/.oxlintrc.json",
            "empty-children/.oxlintrc.json",
            "package-extends/.oxlintrc.json",
            "package-extends/file.ts",
            "string-extends/.oxlintrc.json",
            "string-extends/file.ts",
            "deep/mid/.oxlintrc.json",
            "deep/nested/.oxlintrc.json",
            "deep/nested/file.ts",
            "deep/lost/.oxlintrc.json",
            "deep/lost/file.ts",
            "cycle/.oxlintrc.json",
            "cycle/file.ts",
            "missing/.oxlintrc.json",
            "missing/file.ts",
            "invalid-ancestor/.oxlintrc.json",
            "invalid-ancestor/file.ts",
            "outside/.oxlintrc.json",
            "outside/file.ts",
            "symlink/.oxlintrc.json",
            "symlink/file.ts",
            "multi/.oxlintrc.json",
            "multi/file.ts",
            "multi-ok/.oxlintrc.json",
            "multi-ok/file.ts",
        ],
    )
}

#[test]
fn ancestor_override_subset_flags_lost_and_keeps_valid_nested_configs() {
    let root = fixture_root();
    let findings =
        check_with_files(&root, &config(oxlintrc_yaml()), &oxlintrc_inventory(&root)).unwrap();
    let body = format!("{findings:?}");
    let files: Vec<&str> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    for file in [
        "lost/.oxlintrc.json",
        "drift/.oxlintrc.json",
        "deep/lost/.oxlintrc.json",
        "cycle/.oxlintrc.json",
        "missing/.oxlintrc.json",
        "invalid.json",
        "outside/.oxlintrc.json",
        "symlink/.oxlintrc.json",
        "multi/.oxlintrc.json",
    ] {
        assert!(files.contains(&file), "{body}");
    }
    for file in [
        "nested/.oxlintrc.json",
        "superset/.oxlintrc.json",
        "still-matches/.oxlintrc.json",
        "unmatched/.oxlintrc.json",
        "standalone/.oxlintrc.json",
        "empty-children/.oxlintrc.json",
        "package-extends/.oxlintrc.json",
        "string-extends/.oxlintrc.json",
        "deep/nested/.oxlintrc.json",
        "deep/mid/.oxlintrc.json",
        "multi-ok/.oxlintrc.json",
    ] {
        assert!(!files.contains(&file), "{body}");
    }
    assert_eq!(findings.len(), 9, "{body}");
}

#[test]
fn ancestor_override_subset_reports_cycle_missing_and_outside_paths() {
    let root = fixture_root();
    let findings =
        check_with_files(&root, &config(oxlintrc_yaml()), &oxlintrc_inventory(&root)).unwrap();
    let cycle = findings
        .iter()
        .find(|finding| finding.file == "cycle/.oxlintrc.json")
        .unwrap();
    assert!(cycle.message.contains("extends cycle"), "{cycle:?}");
    let missing = findings
        .iter()
        .find(|finding| finding.file == "missing/.oxlintrc.json")
        .unwrap();
    assert!(missing.message.contains("is missing"), "{missing:?}");
    let outside = findings
        .iter()
        .find(|finding| finding.file == "outside/.oxlintrc.json")
        .unwrap();
    assert!(
        outside.message.contains("outside the repository root"),
        "{outside:?}"
    );
    let symlink = findings
        .iter()
        .find(|finding| finding.file == "symlink/.oxlintrc.json")
        .unwrap();
    assert!(
        symlink.message.contains("outside the repository root"),
        "{symlink:?}"
    );
}

#[test]
fn ancestor_override_subset_keeps_parse_errors_when_a_custom_message_is_set() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[
            "invalid.json",
            "invalid-ancestor/.oxlintrc.json",
            "invalid-ancestor/file.ts",
        ],
    );
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["invalid-ancestor/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
        message: restate lost overrides
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings[0].file, "invalid.json", "{findings:?}");
    assert!(
        findings[0].message.contains("failed to parse JSONC"),
        "{findings:?}"
    );
    assert!(
        !findings[0].message.contains("restate lost overrides"),
        "{findings:?}"
    );
}

#[test]
fn ancestor_override_subset_uses_custom_message_for_subset_failures() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[".oxlintrc.json", "lost/.oxlintrc.json", "lost/file.ts"],
    );
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["lost/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
        message: restate lost overrides
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].message, "restate lost overrides");
}

#[test]
fn ancestor_override_subset_parameterized_keys() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[
            "custom/base.yml",
            "custom/child/config.yml",
            "custom/child/file.ts",
            "custom/child-lost/config.yml",
            "custom/child-lost/file.ts",
        ],
    );
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["custom/**/*.yml"]
    when:
      - key: inherit
    valueAssertions:
      - kind: ancestor-override-subset
        extendsKey: inherit
        overridesKey: layers
        filesKey: globs
        rulesKey: policy
"#,
        ),
        &files,
    )
    .unwrap();
    let body = format!("{findings:?}");
    let found: Vec<&str> = findings
        .iter()
        .map(|finding| finding.file.as_str())
        .collect();
    assert!(found.contains(&"custom/child-lost/config.yml"), "{body}");
    assert!(!found.contains(&"custom/child/config.yml"), "{body}");
    assert!(!found.contains(&"custom/base.yml"), "{body}");
    assert_eq!(findings.len(), 1, "{body}");
}
