use super::{check, fixture_root};

#[test]
fn rejected_names_report_configured_stem_suffixes() {
    // Keep custom suffixes in the diagnostic so a rejected name reveals the
    // configuration-supported mapping (filaments#11499).
    let root = fixture_root("stem-suffix-diagnostics");
    let findings = check(
        &root,
        &crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes.yml"))).unwrap(),
    )
    .unwrap();
    let by_file: std::collections::HashMap<_, _> = findings
        .iter()
        .map(|finding| (finding.file.as_str(), finding.message.as_str()))
        .collect();
    assert_eq!(by_file.len(), 3, "{findings:?}");
    let stripped = by_file["backend/mod/page.mock.test.tsx"];
    assert!(
        stripped.contains("no corresponding source file")
            && stripped.contains("stripped configured stem suffixes: `.mock`"),
        "{stripped}"
    );
    let listed = by_file["backend/mod/orphan.test.tsx"];
    assert!(
        listed.contains("no corresponding source file")
            && listed.contains("configured stemSuffixesToStrip: `.mock`, `.generated`"),
        "{listed}"
    );
    let source = by_file["backend/mod/widget.tsx"];
    assert!(
        source.contains("no corresponding test file")
            && source.contains("configured stemSuffixesToStrip: `.mock`, `.generated`"),
        "{source}"
    );
}
