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

#[test]
fn ancestor_override_subset_reports_cycle_missing_and_outside_paths() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[
            "cycle/.oxlintrc.json",
            "cycle/peer.json",
            "cycle/file.ts",
            "missing/.oxlintrc.json",
            "missing/file.ts",
            "outside/.oxlintrc.json",
            "outside/file.ts",
            "symlink/.oxlintrc.json",
            "symlink/file.ts",
        ],
    );
    let findings = check_with_files(&root, &config(oxlintrc_yaml()), &files).unwrap();
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
