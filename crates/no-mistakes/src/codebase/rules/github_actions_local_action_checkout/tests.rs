use super::scan::check_source;
use crate::config::v2::Unanalyzable;

#[test]
fn checkout_state_is_per_job_and_action_path() {
    let source = include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/unit/.github/workflows/order.yml");
    let value = serde_yaml::from_str(source).unwrap();
    let output = check_source(
        ".github/workflows/order.yml",
        source,
        &value,
        Unanalyzable::Report,
    );
    let lines: Vec<_> = output.findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [6, 22, 28, 33, 40, 43, 47, 51]);
    assert!(output.advisories.is_empty());
    for finding in output.findings {
        assert_eq!(finding.rule, super::RULE_ID);
        assert!(source
            .lines()
            .nth(finding.line - 1)
            .unwrap()
            .contains("uses: ./actions/"));
        assert!(finding
            .message
            .contains("before an unconditional root self checkout"));
    }
}

#[test]
fn literal_root_paths_sparse_ancestors_and_restoration_are_required() {
    let source = include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/unit/.github/workflows/edges.yml");
    let value = serde_yaml::from_str(source).unwrap();
    let output = check_source(
        ".github/workflows/edges.yml",
        source,
        &value,
        Unanalyzable::Report,
    );
    let lines: Vec<_> = output.findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [9, 15, 24, 41]);
    assert!(output.advisories.is_empty());
}

fn diagnostics_fixture() -> (
    std::path::PathBuf,
    crate::config::v2::NoMistakesConfig,
    Vec<std::path::PathBuf>,
) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-cases/rules/github-actions-local-action-checkout/fixture/diagnostics",
        ),
    );
    let config = serde_yaml::from_str(include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/diagnostics/.no-mistakes.yml")).unwrap();
    let files = [
        "suppression.yml",
        "disabled.yml",
        "shape.yml",
        "malformed.yml",
    ]
    .into_iter()
    .map(|file| root.join(".github/workflows").join(file))
    .collect();
    (root, config, files)
}

#[test]
fn diagnostics_honor_suppression_and_unanalyzable_policy() {
    let (root, mut config, files) = diagnostics_fixture();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let raw =
        super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(raw.findings.len(), 6);
    let visible =
        super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert_eq!(visible.findings.len(), 3);
    for (policy, advisories) in [("advise", 2), ("ignore", 0)] {
        config.rules[0].options = serde_yaml::from_str(&format!("unanalyzable: {policy}")).unwrap();
        let output =
            super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                .unwrap();
        assert_eq!(output.findings.len(), 1);
        assert_eq!(output.findings[0].line, 9);
        assert_eq!(output.advisories.len(), advisories);
    }
}

#[test]
fn options_are_strict_and_custom_messages_apply() {
    let (root, mut config, files) = diagnostics_fixture();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    config.rules[0].message = Some("Check out the action first.".to_owned());
    let output =
        super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert_eq!(output.findings.len(), 3);
    assert!(output
        .findings
        .iter()
        .all(|finding| finding.message == "Check out the action first."));
    config.rules[0].options = serde_yaml::from_str("unknownOption: true").unwrap();
    assert!(
        super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).is_err()
    );
}

#[test]
fn flow_quoted_keys_and_block_scalars_keep_authored_uses_lines() {
    for (name, source, lines) in [
        ("forms.yml", include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/locations/.github/workflows/forms.yml"), vec![5, 7, 9]),
        ("root-flow.yml", include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/locations/.github/workflows/root-flow.yml"), vec![2]),
        ("paths.yml", include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/locations/.github/workflows/paths.yml"), vec![7, 18, 19]),
    ] {
        let value = serde_yaml::from_str(source).unwrap();
        let output = check_source(name, source, &value, Unanalyzable::Report);
        assert_eq!(output.findings.iter().map(|finding| finding.line).collect::<Vec<_>>(), lines, "{name}");
    }
}

#[test]
fn standalone_entrypoints_suppress_saved_diagnostics() {
    let (root, config, files) = diagnostics_fixture();
    assert_eq!(super::check(&root, &config).unwrap().len(), 3);
    assert_eq!(
        super::check_with_files(&root, &config, &files)
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn missing_source_obeys_unanalyzable_policy() {
    let (root, mut config, _) = diagnostics_fixture();
    let files = vec![root.join(".github/workflows/missing.yml")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let output =
        super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert_eq!(output.findings.len(), 1);
    assert!(output.findings[0]
        .message
        .contains("cannot read workflow source"));
    config.rules[0].options = serde_yaml::from_str("unanalyzable: advise").unwrap();
    let output =
        super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert!(output.findings.is_empty());
    assert_eq!(output.advisories.len(), 1);
}

#[test]
fn nonstatic_shapes_and_sparse_values_do_not_prove_checkout() {
    let source = include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/boundaries/.github/workflows/shapes.yml");
    let value = serde_yaml::from_str(source).unwrap();
    let output = check_source(
        ".github/workflows/shapes.yml",
        source,
        &value,
        Unanalyzable::Report,
    );
    assert_eq!(
        output
            .findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [16, 20, 24, 28, 32, 36]
    );
}

#[test]
fn reserved_diagnostic_job_name_still_produces_definite_findings() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-local-action-checkout/fixture/named-job"),
    );
    let mut config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/named-job/.no-mistakes.yml")).unwrap();
    let files = vec![root.join(".github/workflows/named.yml")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    for policy in ["report", "advise", "ignore"] {
        config.rules[0].options = serde_yaml::from_str(&format!("unanalyzable: {policy}")).unwrap();
        let output =
            super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                .unwrap();
        assert_eq!(output.findings.len(), 1);
        assert_eq!(output.findings[0].target.as_deref(), Some("unanalyzable#1"));
        assert!(output.advisories.is_empty());
    }
}

#[test]
fn location_collection_tolerates_saved_malformed_and_nonmapping_jobs() {
    for source in [
        include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/diagnostics/.github/workflows/malformed.yml"),
        include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/diagnostics/.github/workflows/shape.yml"),
    ] {
        assert!(super::locations::collect(source).is_empty());
    }
}

#[test]
fn nonworkflow_sources_are_ignored_and_invalid_policy_is_rejected() {
    let (root, mut config, _) = diagnostics_fixture();
    let files = vec![root.join(".no-mistakes.yml")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    config.rules[0].include = vec!["**/*".to_owned(), ".no-mistakes.yml".to_owned()];
    let output =
        super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).unwrap();
    assert!(output.findings.is_empty());
    assert!(output.advisories.is_empty());
    config.rules[0].options = serde_yaml::from_str("unanalyzable: unknown").unwrap();
    assert!(
        super::diagnostics_with_files_and_sources(&root, &config, &files, &sources, false).is_err()
    );
}

#[test]
fn sparse_exclusions_do_not_prove_checkout_or_erase_prior_readiness() {
    let source = include_str!("../../../../../../test-cases/rules/github-actions-local-action-checkout/fixture/exclusions/.github/workflows/exclusions.yml");
    let value = serde_yaml::from_str(source).unwrap();
    let output = check_source(
        ".github/workflows/exclusions.yml",
        source,
        &value,
        Unanalyzable::Report,
    );
    assert_eq!(
        output
            .findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [10]
    );
    assert!(output.advisories.is_empty());
}
