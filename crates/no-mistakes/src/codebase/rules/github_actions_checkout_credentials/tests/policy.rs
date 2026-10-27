use super::*;
use crate::codebase::rules::{analysis_policy, source_store_for_files};
use crate::config::v2::{Effort, NoMistakesConfig, Unanalyzable};

fn fixture() -> (PathBuf, NoMistakesConfig, Vec<PathBuf>) {
    let root = super::tests::fixture_root("unanalyzable");
    let config =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    (root, config, files)
}

#[test]
fn saved_unknown_shapes_obey_each_policy_and_effort_with_suppressions() {
    // Output cases moved to cli_gha_snapshot_checkout_credentials.
    for effort in [Effort::Low, Effort::Max] {
        for policy in [
            Unanalyzable::Report,
            Unanalyzable::Advise,
            Unanalyzable::Ignore,
        ] {
            let (root, mut config, files) = fixture();
            config.rules[0].options = serde_yaml::from_str("{}").unwrap();
            config.rules[0].options["effort"] = serde_yaml::to_value(effort).unwrap();
            config.rules[0].options["unanalyzable"] = serde_yaml::to_value(policy).unwrap();
            let sources = source_store_for_files(&files);
            analysis_policy::prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
            assert_eq!(
                config.analysis_warnings.len(),
                usize::from(effort == Effort::Low)
            );
            let _output =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                    .unwrap();
            // Compare paths as components so this fixture accounting also holds on Windows.
            let workflow_dir = root.join(".github").join("workflows");
            assert_eq!(
                sources.physical_read_count(),
                files
                    .iter()
                    .filter(|path| path.extension().is_some_and(|ext| ext == "yml")
                        && path.parent() == Some(workflow_dir.as_path()))
                    .count()
            );
        }
    }
}

#[test]
fn default_policy_reports_and_deferred_suppression_preserves_both_channels() {
    let (root, mut config, files) = fixture();
    let sources = source_store_for_files(&files);
    let output =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(
        output.findings.len(),
        26,
        "{:?} {:?}",
        output.findings,
        output.advisories
    );
    assert!(output.advisories.is_empty());
    config.rules[0].options = serde_yaml::from_str("unanalyzable: advise").unwrap();
    let output =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(output.findings.len(), 2);
    assert_eq!(output.advisories.len(), 24);
}

#[test]
fn unreadable_sources_use_policy_and_retained_bytes_only_for_suppression() {
    let root = super::tests::fixture_root("unreadable");
    let mut config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap())
            .unwrap();
    let files: Vec<_> = ["invalid.yml", "disabled.yml", "missing.yml"]
        .iter()
        .map(|file| root.join(".github/workflows").join(file))
        .collect();
    let sources = source_store_for_files(&files);
    for deferred in [false, true] {
        for policy in [
            Unanalyzable::Report,
            Unanalyzable::Advise,
            Unanalyzable::Ignore,
        ] {
            config.rules[0].options = serde_yaml::from_str("{}").unwrap();
            config.rules[0].options["unanalyzable"] = serde_yaml::to_value(policy).unwrap();
            let output =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, deferred)
                    .unwrap();
            let count = if deferred { 3 } else { 2 };
            assert_eq!(
                output.findings.len(),
                count * usize::from(policy == Unanalyzable::Report)
            );
            assert_eq!(
                output.advisories.len(),
                count * usize::from(policy == Unanalyzable::Advise)
            );
            for finding in output.findings.iter().chain(&output.advisories) {
                assert!(finding
                    .message
                    .contains("cannot read GitHub Actions source as UTF-8"));
                assert!(finding.message.contains("literal false"));
                assert_eq!(
                    finding.line,
                    if finding.file.ends_with("missing.yml") {
                        1
                    } else {
                        2
                    }
                );
            }
        }
    }
    assert_eq!(sources.physical_read_count(), 3);
}

#[test]
fn invalid_scope_and_analysis_policy_are_errors_instead_of_clean_results() {
    let (root, _, files) = fixture();
    let sources = source_store_for_files(&files);
    for (name, message) in [
        (".no-mistakes.invalid-include.yml", "include"),
        (".no-mistakes.invalid-policy.yml", "unanalyzable"),
    ] {
        let config =
            serde_yaml::from_str(&std::fs::read_to_string(root.join(name)).unwrap()).unwrap();
        let error = diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
            .err()
            .expect("invalid configuration must fail");
        assert!(error.to_string().contains(message), "{error}");
    }
}

#[test]
fn standalone_check_filters_suppressions_without_changing_raw_dispatch() {
    let (root, _, files) = fixture();
    let known_root = super::tests::fixture_root("suppression");
    for (name, visible, raw) in [
        (".no-mistakes.yml", 20, 26),
        (".no-mistakes.advise.yml", 2, 2),
        (".no-mistakes.ignore.yml", 2, 2),
    ] {
        let config =
            serde_yaml::from_str(&std::fs::read_to_string(root.join(name)).unwrap()).unwrap();
        let findings = super::check(&root, &config).unwrap();
        assert_eq!(findings.len(), visible, "{findings:?}");
        assert!(findings
            .iter()
            .all(|finding| !finding.file.ends_with("suppression.yml")
                && !finding.file.ends_with("disabled.yml")
                && !finding.file.ends_with("suppressed.yml")));
        assert_eq!(
            super::check_with_files(&root, &config, &files)
                .unwrap()
                .len(),
            raw
        );
        // The same next-line/file/line directives must also hide definite violations,
        // even when unknown inputs are advised or ignored by this findings-only API.
        let known = super::check(&known_root, &config).unwrap();
        assert_eq!(known.len(), 2, "{known:?}");
        assert!(known
            .iter()
            .any(|finding| finding.file.ends_with("ci.yml") && finding.message.contains("step 3")));
    }
}
