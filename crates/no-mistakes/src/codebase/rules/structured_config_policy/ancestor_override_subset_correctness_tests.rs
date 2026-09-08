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

fn config() -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(
            r#"
policies:
  - files: ["**/.oxlintrc.json"]
    when:
      - key: extends
    valueAssertions:
      - kind: ancestor-override-subset
"#,
        )
        .unwrap(),
        ..Default::default()
    });
    config
}

fn files(root: &Path, rels: &[&str]) -> Vec<PathBuf> {
    rels.iter().map(|rel| root.join(rel)).collect()
}

#[test]
fn ancestor_override_subset_uses_last_matching_rule_value_per_child() {
    let root = fixture_root();
    let findings = check_with_files(
        &root,
        &config(),
        &files(
            &root,
            &[
                "precedence/base.json",
                "precedence/nested/.oxlintrc.json",
                "precedence/nested/file.ts",
            ],
        ),
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn ancestor_override_subset_checks_lost_rules_for_each_child() {
    let root = fixture_root();
    let findings = check_with_files(
        &root,
        &config(),
        &files(
            &root,
            &[
                "per-child/base.json",
                "per-child/nested/.oxlintrc.json",
                "per-child/nested/file.ts",
                "per-child/nested/nested/special/file.ts",
            ],
        ),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "per-child/nested/.oxlintrc.json");
}

#[test]
fn ancestor_override_subset_honors_exclude_files() {
    let root = fixture_root();
    let findings = check_with_files(
        &root,
        &config(),
        &files(
            &root,
            &[
                "exclude/base.json",
                "exclude/ignored/.oxlintrc.json",
                "exclude/ignored/skip.ts",
                "exclude/lost/.oxlintrc.json",
                "exclude/lost/file.ts",
            ],
        ),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "exclude/lost/.oxlintrc.json");
}

#[test]
fn ancestor_override_subset_matches_sibling_globs_from_the_ancestor() {
    let root = fixture_root();
    let findings = check_with_files(
        &root,
        &config(),
        &files(
            &root,
            &[
                "sibling/base/config.json",
                "sibling/app/nested/.oxlintrc.json",
                "sibling/app/nested/file.ts",
            ],
        ),
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "sibling/app/nested/.oxlintrc.json");
}

#[test]
fn ancestor_override_subset_rejects_malformed_extends_and_overrides() {
    let root = fixture_root();
    let findings = check_with_files(
        &root,
        &config(),
        &files(
            &root,
            &[
                "malformed/base.json",
                "malformed/child/.oxlintrc.json",
                "malformed/child/file.ts",
                "malformed/invalid-extends/.oxlintrc.json",
            ],
        ),
    )
    .unwrap();
    let messages = findings
        .iter()
        .map(|finding| &finding.message)
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("files must contain valid string globs")),
        "{findings:?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("extends must be a string")),
        "{findings:?}"
    );
}
