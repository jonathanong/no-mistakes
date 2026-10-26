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
        assert!(
            !raw.findings.is_empty() == has_raw,
            "{name}: unexpected raw findings: {:?}",
            raw.findings
        );
    }
}
