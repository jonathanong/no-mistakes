use crate::codebase::ts_source::VisiblePathSnapshot;
use std::path::Path;

#[test]
fn prepared_fragment_loading_uses_the_shared_snapshot_source_store() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/config-fragments/valid"),
    );
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    super::run_filesystem_rules_with_visible_and_snapshot(&root, None, &[], &snapshot).unwrap();
    assert_eq!(sources.physical_read_count(), 3);
}

#[test]
fn partial_work_lists_do_not_audit_missing_paths_from_a_full_snapshot() {
    let root = crate::test_support::rule_fixture_root("no-empty-or-comments-only-files")
        .join("allow-audit");
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let config = crate::config::v2::load_v2_config_from_source_store(
        &root,
        None,
        &snapshot.paths_for(&root),
        &sources,
    )
    .unwrap();
    let partial = [root.join("clean.ts")];
    let findings =
        super::run_filesystem_rules_with_config_and_snapshot(&root, &config, &partial, &snapshot)
            .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].target.as_deref(), Some("clean.ts"));
    let reads = sources.physical_read_count();
    sources.read_path(&partial[0]).unwrap();
    assert_eq!(sources.physical_read_count(), reads);
    let full = snapshot.paths_for(&root);
    for _ in 0..2 {
        let findings =
            super::run_filesystem_rules_with_config_and_snapshot(&root, &config, &full, &snapshot)
                .unwrap();
        assert_eq!(findings.len(), 2, "{findings:?}");
        assert_eq!(findings[1].target.as_deref(), Some("missing.ts"));
    }
    let reads = sources.physical_read_count();
    super::run_filesystem_rules_with_config_and_snapshot(&root, &config, &full, &snapshot).unwrap();
    assert_eq!(sources.physical_read_count(), reads);
}
