use super::*;
use std::path::Path;

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/structured-config-policy/ancestor-override-subset"),
    )
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
    let mut state = scan::ScanState::default();

    let findings =
        scan::scan_with_state(&root, &opts, &files, &files, &[], &sources, &mut state).unwrap();

    assert_eq!(state.parsed_ancestors.values.len(), 1);
    assert!(state.canonical_inventory.is_some());
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn scan_does_not_initialize_ancestor_state_without_an_active_assertion() {
    let root = fixture_root();
    let files = inventory(&root, &["nested/.oxlintrc.json"]);
    let opts: Options = serde_yaml::from_str(
        r#"
policies:
  - files: ["**/.oxlintrc.json"]
    valueAssertions:
      - kind: boolean
        key: enabled
  - files: ["**/.oxlintrc.json"]
    when:
      - key: absent
    valueAssertions:
      - kind: ancestor-override-subset
"#,
    )
    .unwrap();
    let sources = super::super::source_store_for_files(&files);
    let mut state = scan::ScanState::default();

    let _ = scan::scan_with_state(&root, &opts, &files, &files, &[], &sources, &mut state).unwrap();

    assert!(state.canonical_inventory.is_none());
    assert!(state.parsed_ancestors.values.is_empty());
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
