use super::*;

#[test]
fn deferred_diagnostics_keep_entry_and_delegated_file_findings() {
    for name in ["disable-file", "low-delegate-disabled"] {
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
            !raw.findings.is_empty(),
            "{name}: expected suppressed findings"
        );
    }
}
