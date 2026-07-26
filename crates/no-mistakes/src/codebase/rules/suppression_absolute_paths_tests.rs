use super::*;
use std::path::Path;

#[test]
fn request_sources_only_suppress_registered_absolute_paths() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/suppression/non-inventory");
    let root = fixture.join("request");
    let external = fixture.join("outside.md");
    let sources = source_store_for_files(&[root.join("safe.md")]);
    let mut findings = vec![RuleFinding {
        rule: "my-rule".to_string(),
        file: external.display().to_string(),
        line: 1,
        message: "external".to_string(),
        import: None,
        target: None,
    }];

    suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(
        findings.len(),
        1,
        "unregistered absolute paths stay visible"
    );

    sources.register_trusted_regular_paths(std::slice::from_ref(&external));
    suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty());
}

#[test]
fn portable_absolute_path_detection_rejects_windows_paths_on_every_host() {
    assert!(crate::codebase::ts_source::is_portably_absolute_path(
        Path::new(r"C:\repo\file.md")
    ));
    assert!(crate::codebase::ts_source::is_portably_absolute_path(
        Path::new(r"\\server\share\file.md")
    ));
    assert!(!crate::codebase::ts_source::is_portably_absolute_path(
        Path::new("docs/file.md")
    ));
}
