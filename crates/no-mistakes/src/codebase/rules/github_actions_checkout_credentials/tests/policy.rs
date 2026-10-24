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
            let output =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                    .unwrap();
            assert_eq!(
                output.findings.len(),
                1 + 19 * usize::from(policy == Unanalyzable::Report),
                "{:?} {:?}",
                output.findings,
                output.advisories
            );
            assert_eq!(
                output.advisories.len(),
                19 * usize::from(policy == Unanalyzable::Advise),
                "{:?} {:?}",
                output.findings,
                output.advisories
            );
            if policy != Unanalyzable::Ignore {
                let mut actual: Vec<_> = output
                    .findings
                    .iter()
                    .chain(&output.advisories)
                    .filter(|finding| !finding.file.ends_with("violation.yml"))
                    .map(|finding| {
                        Path::new(&finding.file)
                            .file_stem()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .to_string()
                    })
                    .collect();
                actual.sort();
                let expected = [
                    "invalid",
                    "permissions-caller",
                    "permissions-expression",
                    "permissions-invalid-scope",
                    "permissions-missing",
                    "permissions-null",
                    "permissions-number-key",
                    "permissions-sequence",
                    "permissions-unknown-access",
                    "permissions-unknown-scope",
                    "permissions-unknown-shorthand",
                    "persist-expression",
                    "persist-null",
                    "persist-number",
                    "persist-string",
                    "token-custom",
                    "token-github",
                    "with-expression",
                    "with-sequence",
                ];
                assert_eq!(actual, expected);
            }
            let known = output
                .findings
                .iter()
                .find(|finding| finding.file.ends_with("violation.yml"))
                .unwrap();
            assert!(known
                .message
                .contains("workflow permissions grant contents: write"));
            for finding in output
                .findings
                .iter()
                .chain(&output.advisories)
                .filter(|finding| !finding.file.ends_with("violation.yml"))
            {
                assert!(finding.message.contains(&finding.file));
                assert!(finding.message.contains("literal false"));
                assert!(finding.message.contains("literal read-only permissions"));
                if finding.file.ends_with("invalid.yml") {
                    assert!(finding.message.contains("invalid YAML"));
                    assert!(finding.message.contains("job `unknown` step unknown"));
                } else {
                    assert!(finding.message.contains("job `build` step 1"));
                    assert_eq!(
                        finding.line,
                        if finding.file.ends_with("permissions-missing.yml") {
                            5
                        } else {
                            6
                        }
                    );
                }
            }
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
    assert_eq!(super::check(&root, &config).unwrap(), output.findings);
    config.rules[0].options = serde_yaml::from_str("unanalyzable: advise").unwrap();
    let output =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(output.findings.len(), 1);
    assert_eq!(output.advisories.len(), 25);
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
