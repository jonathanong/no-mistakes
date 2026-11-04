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
