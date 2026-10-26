use super::*;

#[test]
fn deferred_diagnostics_keep_entry_and_delegated_file_findings() {
    for (name, has_raw) in [
        ("disable-file", true),
        ("low-delegate-disabled", true),
        ("deferred-stale", false),
    ] {
        let root = fixture(name);
        let files = fixture_files(&root);
        let sources = super::super::super::source_store_for_files(&files);
        let config = config("{}");
        let visible =
            diagnostics_with_files_and_sources(&root, &config, &files, &sources, None, false)
                .unwrap();
        assert!(
            visible.findings.is_empty(),
            "{name}: {:?}",
            visible.findings
        );
        let raw = diagnostics_with_files_and_sources(&root, &config, &files, &sources, None, true)
            .unwrap();
        assert_eq!(
            !raw.findings.is_empty(),
            has_raw,
            "{name}: unexpected raw findings: {:?}",
            raw.findings
        );
    }
}

#[test]
fn deferred_entry_file_opt_out_keeps_helper_location_and_entry_provenance() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/check/redirect-suppression-entry-disabled"),
    );
    let files = fixture_files(&root);
    let sources = super::super::super::source_store_for_files(&files);
    let diagnostics =
        diagnostics_with_files_and_sources(&root, &config("{}"), &files, &sources, None, true)
            .unwrap();
    assert!(diagnostics.findings.is_empty());
    assert_eq!(diagnostics.suppressed.len(), 1);
    let consumed = &diagnostics.suppressed[0];
    assert_eq!(consumed.file, "next-shared.ts");
    assert_eq!(
        consumed.source_file,
        root.join("next.config.ts").to_string_lossy()
    );
    assert_eq!(consumed.directive.line, 1);
    assert_eq!(
        consumed.directive.kind,
        super::super::super::SuppressionDirectiveKind::File
    );
}
