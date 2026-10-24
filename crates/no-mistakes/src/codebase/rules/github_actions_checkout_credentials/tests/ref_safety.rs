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
    let root = super::tests::fixture_root("ref-safety");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let heads = [
        "custom-token:2",
        "fork-repository:8",
        "head-empty:1",
        "head-list:1",
        "head-mapping:1",
        "head-missing:1",
        "head-read:1",
        "head-write:1",
        "head-write:2",
        "head-write:3",
        "head-write:4",
        "head-write:5",
        "head-write:6",
        "literal-default:3",
        "persisted-write:4",
        "suppression:3",
        "suppression:6",
    ];
    let credentials = [
        "outside-scope:3",
        "persisted-write:1",
        "persisted-write:2",
        "persisted-write:3",
        "persisted-write:4",
        "suppression:6",
        "unknown-suppression:3",
        "unknown-with:1",
        "unknown-with:2",
    ];
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
                assert_eq!(
                    identities(&output.findings, "checks out a pull-request head ref"),
                    heads
                );
                assert_eq!(
                    identities(&output.findings, "uses actions/checkout while"),
                    credentials
                );
                let uncertainty = 27 - 2 * usize::from(configured);
                assert_eq!(
                    output.findings.len(),
                    26 + uncertainty * usize::from(policy == Unanalyzable::Report)
                );
                assert_eq!(
                    output.advisories.len(),
                    uncertainty * usize::from(policy == Unanalyzable::Advise)
                );
                let unknown: Vec<_> = output
                    .findings
                    .iter()
                    .chain(&output.advisories)
                    .filter(|finding| finding.message.contains("cannot analyze the checkout ref"))
                    .cloned()
                    .collect();
                assert_eq!(
                    unknown.len(),
                    (24 - 2 * usize::from(configured))
                        * usize::from(policy != Unanalyzable::Ignore)
                );
                assert!(
                    unknown
                        .iter()
                        .any(|finding| finding.file.ends_with("fork-repository.yml"))
                        || policy == Unanalyzable::Ignore
                );
                for finding in output.findings.iter().chain(&output.advisories) {
                    let text = std::fs::read_to_string(root.join(&finding.file)).unwrap();
                    assert!(text
                        .lines()
                        .nth(finding.line - 1)
                        .unwrap()
                        .contains("uses: actions/checkout"));
                    if finding.message.contains("pull_request_target workflow") {
                        assert!(finding.message.contains("base-side ref"));
                        assert!(finding
                            .message
                            .contains("AND set with.persist-credentials to literal false"));
                    }
                }
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
    let root = super::tests::fixture_root("ref-safety");
    let findings =
        super::check(&root, &config(&root, ".no-mistakes.configured-heads.yml")).unwrap();
    assert_eq!(
        identities(&findings, "checks out a pull-request head ref"),
        ["literal-default:3"]
    );
    assert_eq!(findings.len(), 1);
    assert!(super::check(&root, &config(&root, ".no-mistakes.safe.yml"))
        .unwrap()
        .is_empty());
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

#[test]
fn bare_commit_ids_cannot_be_certified_by_a_default_branch_convention() {
    let root = super::tests::fixture_root("ref-hex-default");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    for label in ["hex40", "hex64", "nonhex40"] {
        for policy in ["report", "advise", "ignore"] {
            let config = config(&root, &format!(".no-mistakes.{label}-{policy}.yml"));
            let sources = source_store_for_files(&files);
            let output =
                diagnostics_with_files_and_sources(&root, &config, &files, &sources, false)
                    .unwrap();
            let risk = usize::from(label != "nonhex40");
            assert_eq!(
                output.findings.len(),
                risk * usize::from(policy == "report")
            );
            assert_eq!(
                output.advisories.len(),
                risk * usize::from(policy == "advise")
            );
            for finding in output.findings.iter().chain(&output.advisories) {
                assert!(finding
                    .message
                    .contains("step 1 cannot analyze the checkout ref"));
                assert!(!finding.message.contains("step 2"));
            }
        }
    }
}
