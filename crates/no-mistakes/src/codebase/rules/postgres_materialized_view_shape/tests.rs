use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

#[test]
fn absent_rule_performs_no_source_reads_without_prepared_facts() {
    let root = crate::test_support::rule_fixture_root(RULE_ID).join("fail");
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(&[]))));
    let findings =
        check_with_files_and_sources(&root, &NoMistakesConfig::default(), &[], &sources).unwrap();
    assert!(findings.is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn standalone_checks_share_catalog_reads() {
    let base = crate::test_support::rule_fixture_root(RULE_ID);
    let root = base.join("fail");
    let config_path = base.parent().unwrap().join("configs/default.yml");
    let config = serde_yaml::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();
    let files = vec![root.join("schema.json")];
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &files,
    ))));
    let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert!(!findings.is_empty());
    let repeated = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert_eq!(findings, repeated);
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn legacy_index_metadata_and_view_paths_keep_distinct_findings() {
    let base = crate::test_support::rule_fixture_root(RULE_ID);
    let config_path = base.parent().unwrap().join("configs/advisory.yml");
    let config = serde_yaml::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();
    for (fixture, count) in [
        ("legacy-index-data", 1),
        ("empty-index-data", 4),
        ("view-read-path", 5),
    ] {
        let root = base.join(fixture);
        let files = vec![root.join("schema.json")];
        let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
            &files,
        ))));
        let findings = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
        assert_eq!(findings.len(), count, "{fixture}: {findings:?}");
        if fixture == "legacy-index-data" {
            assert!(findings[0].target.is_none());
        }
        if fixture == "view-read-path" {
            assert!(findings.iter().any(|finding| finding
                .message
                .contains("records via view wrapper_view -> view some_view;")));
        }
        assert_eq!(sources.physical_read_count(), 1);
    }
}
