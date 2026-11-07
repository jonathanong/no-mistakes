use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

#[test]
fn absent_rule_performs_no_source_reads_without_prepared_facts() {
    let root = crate::test_support::rule_fixture_root(RULE_ID).join("fail");
    let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(&[]))));
    let output = diagnostics_with_sources_and_facts(
        &root,
        &NoMistakesConfig::default(),
        &[],
        &sources,
        None,
        false,
    )
    .unwrap();
    assert!(output.findings.is_empty());
    assert!(output.advisories.is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn standalone_diagnostics_share_catalog_reads_and_apply_file_suppression() {
    let base = crate::test_support::rule_fixture_root(RULE_ID);
    for (fixture, config_name, suppressed) in [
        ("fail", "default.yml", false),
        ("suppressed", "suppressed.yml", true),
    ] {
        let root = base.join(fixture);
        let config_path = base.parent().unwrap().join("configs").join(config_name);
        let config = serde_yaml::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();
        let files = vec![root.join("schema.json")];
        let sources = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
            &files,
        ))));
        let output =
            diagnostics_with_sources_and_facts(&root, &config, &files, &sources, None, false)
                .unwrap();
        assert!(output.findings.is_empty());
        if !suppressed {
            let raw = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
            assert_eq!(raw, output.advisories);
        }
        assert_eq!(output.advisories.is_empty(), suppressed);
        assert_eq!(sources.physical_read_count(), 1);
    }
}
