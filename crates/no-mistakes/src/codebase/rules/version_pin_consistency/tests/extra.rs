use super::*;

// Output cases moved to the cli_pkgcfg_snapshot modules.

#[test]
fn deferred_suppression_still_emits_disabled_file() {
    let root = fixture("disable-file");
    let files: Vec<PathBuf> = PAIR.iter().map(|file| root.join(file)).collect();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let findings = check_with_files_sources_and_deferred_suppression(
        &root,
        &config(OPTIONS),
        &files,
        &sources,
        true,
    )
    .unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].message.contains("version mismatch"),
        "{findings:?}"
    );
}
