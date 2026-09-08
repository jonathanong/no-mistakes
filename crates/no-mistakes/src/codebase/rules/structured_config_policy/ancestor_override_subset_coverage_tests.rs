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

#[test]
fn scan_caches_a_common_ancestor_across_nested_configs() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[
            ".oxlintrc.json",
            "nested/.oxlintrc.json",
            "nested/file.ts",
            "string-extends/.oxlintrc.json",
            "string-extends/file.ts",
        ],
    );
    let opts: Options = serde_yaml::from_str(
        r#"
policies:
  - files: ["**/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
"#,
    )
    .unwrap();
    let sources = super::super::source_store_for_files(&files);
    let mut cache = ancestor_override_subset::ParsedAncestorCache::default();

    let findings =
        scan::scan_with_parsed_ancestors(&root, &opts, &files, &files, &[], &sources, &mut cache)
            .unwrap();

    assert_eq!(
        ancestor_override_subset::parsed_ancestor_parse_count(&cache, &root.join(".oxlintrc.json"),),
        1
    );
    assert!(findings
        .iter()
        .any(|finding| finding.file == "nested/.oxlintrc.json"));
    assert!(findings
        .iter()
        .any(|finding| finding.file == "string-extends/.oxlintrc.json"));
}

#[test]
fn ancestor_override_subset_rejects_malformed_overrides_and_keeps_valid_configs() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[
            "diamond/shared.json",
            "diamond/left.json",
            "diamond/right.json",
            "diamond/nested/.oxlintrc.json",
            "diamond/nested/file.ts",
            "odd-parent.json",
            "odd/.oxlintrc.json",
            "odd/file.ts",
            "bool-extends/.oxlintrc.json",
            "abs/.oxlintrc.json",
            // Oxlint `*` does not match `/`. `star-seg/*.ts` must not treat
            // `star-seg/inner/file.ts` as a lost ancestor override.
            "star-seg-parent.json",
            "star-seg/.oxlintrc.json",
            "star-seg/inner/file.ts",
        ],
    );
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["**/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
        key: rules
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
    assert!(!found.contains(&"diamond/nested/.oxlintrc.json"), "{body}");
    assert!(found.contains(&"odd/.oxlintrc.json"), "{body}");
    assert!(found.contains(&"bool-extends/.oxlintrc.json"), "{body}");
    assert!(!found.contains(&"star-seg/.oxlintrc.json"), "{body}");
    assert!(found.contains(&"abs/.oxlintrc.json"), "{body}");
    assert_eq!(findings.len(), 3, "{body}");
    assert!(
        findings
            .iter()
            .any(|finding| finding.file == "abs/.oxlintrc.json"
                && finding.message.contains("portable relative path")),
        "{body}"
    );
}

#[test]
fn ancestor_override_subset_uses_assertion_key_as_finding_target() {
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
        key: rules
"#,
        ),
        &files,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("rules"));
}

#[test]
fn ancestor_override_subset_uses_in_scope_children_when_include_is_configs_only() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[".oxlintrc.json", "lost/.oxlintrc.json", "lost/file.ts"],
    );
    let mut config = config(
        r#"
policies:
  - files: ["lost/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
"#,
    );
    config.rules[0].include = vec!["**/.oxlintrc.json".to_string()];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].file, "lost/.oxlintrc.json");
}

#[test]
fn ancestor_override_subset_fails_closed_for_unresolvable_roots_and_nested_paths() {
    let fixture = fixture_root();
    let value = serde_yaml::from_str::<Value>(r#"{"extends":"../.oxlintrc.json"}"#).unwrap();
    let assertion = ValueAssertion::default();
    let missing_root = fixture.join("missing-root");
    let sources = super::super::source_store_for_files(&[]);

    let missing_root_inventory = paths::CanonicalInventory::new(&missing_root, &[]);
    let mut missing_root_cache = ancestor_override_subset::ParsedAncestorCache::default();
    let missing_root_findings = ancestor_override_subset::check_ancestor_override_subset(
        &missing_root.join("nested/.oxlintrc.json"),
        "nested/.oxlintrc.json",
        &sources,
        &value,
        &assertion,
        &missing_root_inventory,
        &mut missing_root_cache,
    );
    assert_eq!(missing_root_findings.len(), 1);
    assert!(missing_root_findings[0]
        .message
        .contains("cannot resolve the repository root safely"));

    let outside_nested = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let outside_inventory = paths::CanonicalInventory::new(&fixture, &[]);
    let mut outside_cache = ancestor_override_subset::ParsedAncestorCache::default();
    let outside_findings = ancestor_override_subset::check_ancestor_override_subset(
        &outside_nested,
        "outside/.oxlintrc.json",
        &sources,
        &value,
        &assertion,
        &outside_inventory,
        &mut outside_cache,
    );
    assert_eq!(outside_findings.len(), 1);
    assert!(outside_findings[0]
        .message
        .contains("nested config is outside the repository root"));
}

#[test]
fn ancestor_override_subset_reports_each_malformed_override_shape() {
    let root = fixture_root();
    let files = inventory(
        &root,
        &[
            "malformed-shapes/base.json",
            "malformed-shapes/nested/.oxlintrc.json",
            "malformed-shapes/nested/file.ts",
        ],
    );
    let findings = check_with_files(
        &root,
        &config(
            r#"
policies:
  - files: ["malformed-shapes/nested/.oxlintrc.json"]
    valueAssertions:
      - kind: ancestor-override-subset
"#,
        ),
        &files,
    )
    .unwrap();
    let messages = findings
        .iter()
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>();
    for detail in [
        "must be an object",
        "rules must be an object",
        "excludeFiles must contain valid string globs",
    ] {
        assert!(
            messages.iter().any(|message| message.contains(detail)),
            "{findings:?}"
        );
    }
    assert_eq!(findings.len(), 3, "{findings:?}");
}
