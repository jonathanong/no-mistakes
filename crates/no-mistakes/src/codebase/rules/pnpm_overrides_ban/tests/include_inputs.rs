use super::*;

#[test]
fn include_helpers_leave_root_parse_errors_and_nonlocal_sources_to_the_resolver() {
    let root = fixture_root("unanalyzable-includes");
    let malformed = fixture_root("malformed-workspace").join("pnpm-workspace.yaml");
    let unsupported = root.join("unsupported-sources.yaml");
    let sources =
        crate::codebase::rules::source_store_for_files(&[malformed.clone(), unsupported.clone()]);
    for path in [&malformed, &unsupported] {
        let text = sources.read_path(path).unwrap();
        let (findings, failures) = unavailable_includes::collect(&root, &text, &sources, &[]);
        assert!(findings.is_empty());
        assert!(failures.is_empty());
        // Only the workspace document owns include declarations.
        assert!(included::included_override_findings(&root, path, &text, &sources, &[]).is_empty());
    }
    assert_eq!(sources.physical_read_count(), 2);
}
