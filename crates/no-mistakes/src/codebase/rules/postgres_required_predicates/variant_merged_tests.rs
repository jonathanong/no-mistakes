use super::super::*;

#[test]
fn merged_variant_requirements_keep_helper_attribution_and_caller_suppression_separate() {
    let root = super::fixture("variant-merged-suppression");
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files = [root.join("src/caller.ts"), root.join("src/helper.ts")];
    let mut findings = check_with_files(&root, &config, &files).unwrap();
    // Keep both caller-side requirements for suppression accounting before filtering.
    assert_eq!(findings.len(), 5, "{findings:#?}");
    let sources = super::super::super::source_store_for_files(&files);
    super::super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert_eq!(findings.len(), 3, "{findings:#?}");
    let local = &findings[0];
    assert_eq!(local.file, "src/caller.ts");
    assert_eq!(local.line, 5);
    assert!(local.import.is_none(), "{local:#?}");
    for column in ["tenant_id", "region_id"] {
        assert_eq!(local.message.matches(column).count(), 1, "{local:#?}");
    }
    // Foreign SQL without a physical byte offset retains one finding per requirement.
    let mut imports = Vec::new();
    for finding in &findings[1..] {
        assert_eq!(finding.file, "src/helper.ts");
        assert_eq!(finding.line, 2);
        imports.push(finding.import.as_deref().unwrap());
    }
    imports.sort_unstable();
    assert_eq!(imports, ["accounts.region_id", "accounts.tenant_id"]);
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() == Some("accounts")));
    let checked =
        super::super::super::run_filesystem_rules(&root, Some(&root.join(".no-mistakes.yml")))
            .unwrap();
    assert_eq!(checked, findings);
}
