use super::*;

#[test]
fn swift_darwin_imports_report_unreadable_selected_sources() {
    let fixture = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/swift-no-unguarded-darwin-import"),
    );
    let root = fixture.join("fixture");
    let config = crate::config::v2::load_v2_config(&root, Some(&fixture.join("configs/check.yml")))
        .unwrap();
    let files = [
        root.join("Tests/ClockTests.swift"),
        root.join("Tests/ClockTests.swift/Unreadable.swift"),
    ];
    let findings = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert_eq!(findings[0].file, "Tests/ClockTests.swift");
    assert_eq!(findings[0].line, 2);
    assert_eq!(findings[1].file, "Tests/ClockTests.swift/Unreadable.swift");
    assert_eq!(findings[1].target.as_deref(), Some("unanalyzable"));
    assert!(findings[1].message.contains("cannot read source"));
}
