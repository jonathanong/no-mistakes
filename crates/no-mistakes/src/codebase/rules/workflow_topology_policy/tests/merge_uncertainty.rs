use super::{config, fixture};
use crate::codebase::ts_source::VisiblePathSnapshot;
use crate::config::v2::Unanalyzable;

#[test]
fn prepared_diagnostics_apply_or_defer_suppression_for_both_channels() {
    let root = fixture("merge-location-suppressed");
    let snapshot = VisiblePathSnapshot::new(&root);
    let sources = snapshot.source_store_for(&root);
    let files = snapshot.paths_for(&root);
    let mut config = config("forbiddenEvents: [pull_request_target]");
    config.unanalyzable = Some(Unanalyzable::Advise);
    let inputs = super::super::prepare::from_files(&root, &config, &files, &sources);
    let diagnostics = |defer_suppression| {
        super::super::diagnostics_prepared(
            &root,
            &config,
            None,
            &sources,
            &inputs,
            None,
            defer_suppression,
        )
        .unwrap()
    };

    let unsuppressed = diagnostics(true);
    assert_eq!(unsuppressed.findings.len(), 1);
    assert_eq!(unsuppressed.advisories.len(), 1);
    let suppressed = diagnostics(false);
    assert!(suppressed.findings.is_empty());
    assert!(suppressed.advisories.is_empty());
}
