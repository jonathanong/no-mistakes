// Output moved to cli_mrf_snapshot_file_extension_policy.

#[test]
fn stale_allowances_require_checked_files_and_raw_findings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/rules/unused-allow-other"),
    );
    let config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap();
    let files =
        crate::codebase::ts_source::discover_files(&root, &config.filesystem.skip_directories);
    let findings = super::check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].rule, "unused-allow-entry");
    let findings = super::check_with_files(&root, &config, &[]).unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}
