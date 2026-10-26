use super::*;

#[test]
fn checkout_unknowns_reach_aggregate_findings_and_advisories_once() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-checkout-credentials/fixture/unanalyzable");
    for (name, failures, advisory_count) in [
        (".no-mistakes.yml", 20, 0),
        (".no-mistakes.advise.yml", 2, 18),
        (".no-mistakes.ignore.yml", 2, 0),
    ] {
        let results = crate::test_support::without_hygiene_advisories(
            run_all(root.clone(), Some(root.join(name)), None).unwrap(),
        );
        assert_eq!(results.rules.len(), failures, "{:?}", results.rules);
        assert_eq!(
            results.advisories.len(),
            advisory_count,
            "{:?}",
            results.advisories
        );
        assert!(results
            .rules
            .iter()
            .any(|finding| finding.file.ends_with("violation.yml")));
        assert!(results
            .advisories
            .iter()
            .all(|finding| finding.rule == "github-actions-checkout-credentials"));
    }
}

#[test]
fn trusted_target_ref_and_credential_diagnostics_reach_aggregate_once() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/github-actions-checkout-credentials/fixture/ref-safety");
    for (name, failures, advisory_count) in [
        (".no-mistakes.yml", 53, 0),
        (".no-mistakes.max-advise.yml", 26, 27),
        (".no-mistakes.max-ignore.yml", 26, 0),
    ] {
        let results = crate::test_support::without_hygiene_advisories(
            run_all(root.clone(), Some(root.join(name)), None).unwrap(),
        );
        assert_eq!(results.rules.len(), failures, "{:?}", results.rules);
        assert_eq!(
            results.advisories.len(),
            advisory_count,
            "{:?}",
            results.advisories
        );
        // Both risks on a single checkout remain separate; uncertainty never hides credentials.
        assert!(results
            .rules
            .iter()
            .any(|finding| finding.file.ends_with("persisted-write.yml")
                && finding.message.contains("uses actions/checkout while")));
        assert!(results
            .rules
            .iter()
            .any(|finding| finding.file.ends_with("head-read.yml")
                && finding
                    .message
                    .contains("checks out a pull-request head ref")));
    }
}
