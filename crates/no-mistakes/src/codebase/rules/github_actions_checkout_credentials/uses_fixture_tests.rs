use crate::config::v2::NoMistakesConfig;
use std::path::PathBuf;

#[test]
fn checkout_uses_variants_are_classified_from_saved_workflow() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-checkout-credentials/fixture/3304-uses"),
    );
    let config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let workflow = root.join(".github/workflows/ci.yml");
    let files = vec![workflow];
    let findings = super::check_with_files(&root, &config, &files).unwrap();

    assert_eq!(findings.len(), 4, "{findings:?}");
    let mut targets: Vec<_> = findings
        .iter()
        .map(|finding| finding.target.as_deref().unwrap())
        .collect();
    targets.sort_unstable();
    assert_eq!(targets, ["case", "comment", "quoted", "slash"]);
    assert!(findings
        .iter()
        .all(|finding| finding.file == ".github/workflows/ci.yml"));
}
