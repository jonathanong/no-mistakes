use super::*;
use crate::codebase::rules::{analysis_policy, source_store_for_files};
use crate::config::v2::{Effort, NoMistakesConfig, Unanalyzable};

fn config(root: &Path, name: &str) -> NoMistakesConfig {
    serde_yaml::from_str(&std::fs::read_to_string(root.join(name)).unwrap()).unwrap()
}

fn identities(findings: &[RuleFinding], needle: &str) -> Vec<String> {
    let mut result: Vec<_> = findings
        .iter()
        .filter(|finding| finding.message.contains(needle))
        .map(|finding| {
            let file = Path::new(&finding.file)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            let step = finding
                .message
                .split("step ")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .trim_end_matches(':');
            format!("{file}:{step}")
        })
        .collect();
    result.sort();
    result
}

#[test]
fn saved_refs_are_independent_of_credentials_across_policies_efforts_and_suppressions() {
    // Output cases moved to cli_gha_snapshot_checkout_credentials.
    let root = super::tests::fixture_root("ref-safety");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    for effort in [Effort::Max, Effort::Low] {
        for policy in [
            Unanalyzable::Report,
            Unanalyzable::Advise,
            Unanalyzable::Ignore,
        ] {
            for configured in [false, true] {
                let suffix = match policy {
                    Unanalyzable::Report => "report",
                    Unanalyzable::Advise => "advise",
                    Unanalyzable::Ignore => "ignore",
                };
                let name = if configured && effort == Effort::Low {
                    format!(".no-mistakes.configured-low-{suffix}.yml")
                } else if configured {
                    format!(".no-mistakes.configured-{suffix}.yml")
                } else if effort == Effort::Low {
                    format!(".no-mistakes.low-{suffix}.yml")
                } else if policy == Unanalyzable::Report {
                    ".no-mistakes.yml".to_string()
                } else {
                    format!(".no-mistakes.max-{suffix}.yml")
                };
                let mut config = config(&root, &name);
                let sources = source_store_for_files(&files);
                analysis_policy::prepare_rule_analysis(&root, &mut config, &files, &sources)
                    .unwrap();
                assert_eq!(
                    config.analysis_warnings.len(),
                    usize::from(effort == Effort::Low)
                );
                let output =
                    diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                        .unwrap();
                for finding in output.findings.iter().chain(&output.advisories) {
                    let text = std::fs::read_to_string(root.join(&finding.file)).unwrap();
                    assert!(
                        text.lines()
                            .nth(finding.line - 1)
                            .unwrap()
                            .contains("uses: actions/checkout"),
                        "{finding:?}"
                    );
                }
                let uncertainty = 27 - 2 * usize::from(configured);
                let raw =
                    diagnostics_with_files_and_sources(&root, &config, &files, &sources, true)
                        .unwrap();
                assert_eq!(
                    identities(&raw.findings, "checks out a pull-request head ref").len(),
                    23
                );
                assert_eq!(
                    identities(&raw.findings, "uses actions/checkout while").len(),
                    14
                );
                assert_eq!(
                    raw.findings.len(),
                    37 + (uncertainty + 2) * usize::from(policy == Unanalyzable::Report)
                );
                assert_eq!(
                    raw.advisories.len(),
                    (uncertainty + 2) * usize::from(policy == Unanalyzable::Advise)
                );
                assert_eq!(sources.physical_read_count(), 19);
                // Standalone checks filter both obligations on the same checkout line.
                assert_eq!(
                    super::check(&root, &config).unwrap().len(),
                    output.findings.len()
                );
            }
        }
    }
}

#[test]
fn saved_default_branch_conventions_are_explicit_and_cannot_whitelist_pr_refs() {
    // Output cases moved to cli_gha_snapshot_checkout_credentials.
    let root = super::tests::fixture_root("ref-safety");
    for suffix in [
        "empty",
        "expression",
        "heads-empty",
        "pull-root",
        "pull",
        "type",
    ] {
        let name = format!(".no-mistakes.invalid-default-{suffix}.yml");
        let error = super::check(&root, &config(&root, &name))
            .unwrap_err()
            .to_string();
        assert!(error.contains("defaultBranch"), "{error}");
        if suffix != "type" {
            assert!(
                error.contains("nonempty literal base branch name"),
                "{error}"
            );
        }
    }
}

// Output cases moved to cli_gha_snapshot_checkout_credentials.
