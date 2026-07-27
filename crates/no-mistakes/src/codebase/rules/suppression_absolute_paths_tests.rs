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

    sources.register_trusted_regular_paths(
        std::slice::from_ref(&external),
        std::slice::from_ref(&fixture),
    );
    suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty());
}

#[test]
#[cfg(unix)]
fn registered_external_symlink_must_resolve_within_its_trusted_root() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/rules/filesystem-dispatch/markdown-external-project");
    let request = fixture.join("request");
    let external = fixture.join("external");
    let escaped = external.join("escaped-suppressed.md");
    let sources = source_store_for_files(&[request.join("CLAUDE.md")]);
    sources.register_trusted_regular_paths(
        std::slice::from_ref(&escaped),
        std::slice::from_ref(&external),
    );
    let mut findings = vec![RuleFinding {
        rule: MARKDOWN_REACHABILITY.to_string(),
        file: escaped.display().to_string(),
        line: 1,
        message: "escaped symlink".to_string(),
        import: None,
        target: None,
    }];

    suppress_rule_findings_with_sources(&request, &mut findings, &sources);

    assert_eq!(
        findings.len(),
        1,
        "a symlink must not import suppression from outside its configured project"
    );
}

#[test]
fn portable_absolute_path_detection_accepts_windows_paths_on_every_host() {
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
