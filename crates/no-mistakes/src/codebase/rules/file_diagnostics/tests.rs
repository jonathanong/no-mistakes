use super::*;
use crate::codebase::{check_facts, ts_source::SourceStore};
use crate::config::v2::{Effort, Unanalyzable};
use std::{path::PathBuf, sync::Arc};

const CASES: &[(&str, usize)] = &[
    ("rust-max-lines-per-file", 1),
    ("rust-no-inline-allows", 2),
    ("rust-no-inline-tests", 2),
    ("csharp-max-lines-per-file", 1),
    ("no-empty-or-comments-only-files", 1),
    ("no-raw-ephemeral-port", 1),
    ("structured-config-policy", 1),
    ("github-actions-pinned-hash", 1),
    ("server-route-client-boundary", 2),
];

fn saved() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&crate::test_support::rule_fixture_root(
        "file-input-analysis",
    ))
}

fn config(root: &Path, rule: &str) -> NoMistakesConfig {
    serde_yaml::from_str(&std::fs::read_to_string(root.join(format!("{rule}.yml"))).unwrap())
        .unwrap()
}

fn files(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    config.rules[0]
        .include
        .iter()
        .map(|path| root.join(path))
        .collect()
}

fn evaluate(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &Arc<SourceStore>,
    defer: bool,
) -> AnalysisDiagnostics {
    if config.rules[0].rule == "server-route-client-boundary" {
        let facts = check_facts::collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            files.to_vec(),
            Vec::new(),
            check_facts::CheckFactPlan {
                server_route_client_boundary: true,
                ..Default::default()
            },
            None,
            Arc::clone(sources),
        );
        super::super::server_route_client_boundary::diagnostics_with_facts_for_aggregate(
            root, config, &facts, sources, None, defer,
        )
        .unwrap()
    } else {
        diagnostics_with_files_and_sources(root, config, files, sources, defer).unwrap()
    }
}

#[test]
fn file_input_policy_preserves_definite_findings_at_every_supported_effort() {
    let materialized = crate::test_support::materialize_saved_fixture(&saved());
    let root = materialized.path();
    for &(rule, uncertain) in CASES {
        for effort in [Effort::Max, Effort::Low] {
            for (policy, failures, advisories) in [
                (Unanalyzable::Report, uncertain + 1, 0),
                (Unanalyzable::Advise, 1, uncertain),
                (Unanalyzable::Ignore, 1, 0),
            ] {
                let mut config = config(root, rule);
                config.effort = Some(effort);
                config.unanalyzable = Some(policy);
                let files = files(root, &config);
                let sources = super::super::source_store_for_files(&files);
                let output = evaluate(root, &config, &files, &sources, false);
                assert_eq!(
                    output.findings.len(),
                    failures,
                    "{rule} {effort:?} {policy:?}: {:?}",
                    output.findings
                );
                assert_eq!(
                    output.advisories.len(),
                    advisories,
                    "{rule} {effort:?} {policy:?}: {:?}",
                    output.advisories
                );
                assert!(output
                    .findings
                    .iter()
                    .chain(&output.advisories)
                    .all(|finding| !finding.file.contains("suppressed")));
                for finding in output
                    .findings
                    .iter()
                    .chain(&output.advisories)
                    .filter(|finding| finding.target.as_deref() == Some("unanalyzable"))
                {
                    if finding.file.contains("/bad.") {
                        assert!(
                            finding.message.contains("restore read access"),
                            "{rule}: {}",
                            finding.message
                        );
                        assert!(!finding.message.contains("repair source syntax"), "{rule}");
                    } else if finding.file.contains("malformed") {
                        assert!(
                            finding.message.contains("repair source syntax"),
                            "{rule}: {}",
                            finding.message
                        );
                    }
                }
                assert_eq!(sources.physical_read_count(), files.len(), "{rule}");
            }
        }
    }
}

#[test]
fn file_input_missing_candidates_and_custom_messages_use_application_policy() {
    let root = saved();
    for &(rule, _) in CASES {
        for policy in ["report", "advise", "ignore"] {
            let mut config = config(&root, rule);
            let selected = config.rules[0].include[0].replace("bad.", "missing.");
            config.rules[0].include = vec![selected];
            config.rules[0].options["unanalyzable"] = policy.into();
            config.rules[0].message = Some(format!("repair required input for {rule}"));
            let files = files(&root, &config);
            let sources = super::super::source_store_for_files(&files);
            let output = evaluate(&root, &config, &files, &sources, false);
            assert_eq!(
                output.findings.len(),
                usize::from(policy == "report"),
                "{rule}"
            );
            assert_eq!(
                output.advisories.len(),
                usize::from(policy == "advise"),
                "{rule}"
            );
            for finding in output.findings.iter().chain(&output.advisories) {
                assert_eq!(finding.line, 1);
                assert_eq!(finding.message, config.rules[0].message.as_deref().unwrap());
                assert_eq!(finding.target.as_deref(), Some("unanalyzable"));
            }
            assert_eq!(sources.physical_read_count(), 1, "{rule}");
        }
    }
}

#[test]
fn file_input_deferred_suppression_keeps_invalid_utf8_for_audit() {
    let root = saved();
    for &(rule, uncertain) in CASES {
        let config = config(&root, rule);
        let files = files(&root, &config);
        let sources = super::super::source_store_for_files(&files);
        let output = evaluate(&root, &config, &files, &sources, true);
        assert_eq!(
            output.findings.len(),
            uncertain + 2,
            "{rule}: {:?}",
            output.findings
        );
        let suppressed = output
            .findings
            .iter()
            .find(|finding| finding.file.contains("suppressed"))
            .unwrap();
        assert_eq!(
            suppressed.line, 2,
            "location comes from the retained invalid-byte prefix: {rule}"
        );
        assert_eq!(sources.physical_read_count(), files.len(), "{rule}");
    }
}

#[test]
fn file_input_exclusions_and_overlapping_policies_remain_application_local() {
    let root = saved();
    for &(rule, uncertain) in CASES {
        let mut config = config(&root, rule);
        let mut advice = config.rules[0].clone();
        advice.options["unanalyzable"] = "advise".into();
        advice.message = Some("advisory policy".into());
        config.rules[0].message = Some("failure policy".into());
        config.rules.push(advice);
        let files = files(&root, &config);
        let sources = super::super::source_store_for_files(&files);
        let output = evaluate(&root, &config, &files, &sources, false);
        assert_eq!(
            output.findings.len(),
            uncertain + 2,
            "{rule}: {:?}",
            output.findings
        );
        assert_eq!(
            output.advisories.len(),
            uncertain,
            "{rule}: {:?}",
            output.advisories
        );
        assert!(output
            .advisories
            .iter()
            .all(|finding| finding.message == "advisory policy"));
        assert_eq!(sources.physical_read_count(), files.len(), "{rule}");
        config.rules.truncate(1);
        config.rules[0].exclude = config.rules[0].include.clone();
        let output = evaluate(&root, &config, &files, &sources, false);
        assert!(
            output.findings.is_empty() && output.advisories.is_empty(),
            "{rule}"
        );
    }
}

#[test]
fn file_input_standalone_entrypoints_preserve_uncertainty_policy() {
    // Discovery returns normalized paths even when the caller spells the root with a dot.
    let root = saved().join(".");
    type Check = fn(&Path, &NoMistakesConfig) -> anyhow::Result<Vec<RuleFinding>>;
    let checks: &[(&str, usize, Check)] = &[
        (
            "rust-max-lines-per-file",
            1,
            super::super::rust_max_lines_per_file::check,
        ),
        (
            "rust-no-inline-allows",
            2,
            super::super::rust_no_inline_allows::check,
        ),
        (
            "rust-no-inline-tests",
            2,
            super::super::rust_no_inline_tests::check,
        ),
        (
            "csharp-max-lines-per-file",
            1,
            super::super::csharp_max_lines_per_file::check,
        ),
        (
            "no-empty-or-comments-only-files",
            1,
            super::super::no_empty_or_comments_only_files::check,
        ),
        (
            "github-actions-pinned-hash",
            1,
            super::super::github_actions_pinned_hash::check,
        ),
        (
            "server-route-client-boundary",
            2,
            super::super::server_route_client_boundary::check,
        ),
    ];
    for &(rule, uncertain, check) in checks {
        for policy in [
            Unanalyzable::Report,
            Unanalyzable::Advise,
            Unanalyzable::Ignore,
        ] {
            let mut config = config(&root, rule);
            config.unanalyzable = Some(policy);
            let findings = check(&root, &config).unwrap();
            assert_eq!(
                findings.len(),
                1 + usize::from(policy == Unanalyzable::Report) * uncertain,
                "{rule}: {findings:?}"
            );
        }
    }
}

#[test]
fn malformed_workflow_keeps_proven_pin_failures_under_every_policy() {
    let root = saved();
    for (policy, failures, advisories) in [
        (Unanalyzable::Report, 2, 0),
        (Unanalyzable::Advise, 1, 1),
        (Unanalyzable::Ignore, 1, 0),
    ] {
        let mut config = config(&root, "github-actions-pinned-hash");
        config.unanalyzable = Some(policy);
        config.rules[0].include = vec![".github/workflows/malformed.yml".into()];
        let files = files(&root, &config);
        let sources = super::super::source_store_for_files(&files);
        let output = evaluate(&root, &config, &files, &sources, false);
        assert_eq!(output.findings.len(), failures, "{:?}", output.findings);
        assert_eq!(output.advisories.len(), advisories);
        assert_eq!(sources.physical_read_count(), 1);
        assert_eq!(sources.yaml_parse_count(), 1);
    }
}

#[test]
fn required_key_named_unanalyzable_is_still_a_definite_config_violation() {
    let root = saved();
    for policy in [Unanalyzable::Advise, Unanalyzable::Ignore] {
        let mut config = config(&root, "structured-config-policy");
        config.unanalyzable = Some(policy);
        config.rules[0].include = vec!["structured/definite.json".into()];
        config.rules[0].options["policies"][0]["requiredKeys"] =
            serde_yaml::to_value(["unanalyzable"]).unwrap();
        let files = files(&root, &config);
        let sources = super::super::source_store_for_files(&files);
        let output = evaluate(&root, &config, &files, &sources, false);
        assert_eq!(output.findings.len(), 1);
        assert!(output.advisories.is_empty());
        assert_eq!(output.findings[0].target.as_deref(), Some("unanalyzable"));
    }
}

#[test]
fn unsupported_port_file_does_not_request_source() {
    let root = saved();
    let mut config = config(&root, "no-raw-ephemeral-port");
    config.rules[0].include = vec!["ports/ignored.txt".into()];
    config.rules[0].options["include"] = serde_yaml::to_value(["ports/*.txt"]).unwrap();
    let files = files(&root, &config);
    let sources = super::super::source_store_for_files(&files);
    let output = evaluate(&root, &config, &files, &sources, false);
    assert!(output.findings.is_empty() && output.advisories.is_empty());
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn explicit_file_rule_exemptions_do_not_read_unavailable_inputs() {
    let root = saved();
    for &(rule, _) in CASES
        .iter()
        .filter(|(rule, _)| *rule != "server-route-client-boundary")
    {
        let mut config = config(&root, rule);
        match rule {
            "no-empty-or-comments-only-files" => {
                config.rules[0].options["intentionallyEmpty"] =
                    serde_yaml::to_value(&config.rules[0].include).unwrap();
            }
            "no-raw-ephemeral-port" => {
                config.rules[0].options["allow"] = serde_yaml::to_value(["**"]).unwrap()
            }
            "github-actions-pinned-hash" => {
                config.rules[0].options["excludePaths"] =
                    serde_yaml::to_value([".github/**"]).unwrap()
            }
            "structured-config-policy" => {
                config.rules[0].options["policies"] =
                    serde_yaml::to_value(Vec::<String>::new()).unwrap()
            }
            _ => {
                let prefix = config.rules[0].include[0]
                    .split('/')
                    .next()
                    .unwrap()
                    .to_string();
                config.rules[0].options["excludes"] = serde_yaml::to_value([prefix]).unwrap();
            }
        }
        let files = files(&root, &config);
        let sources = super::super::source_store_for_files(&files);
        let output = evaluate(&root, &config, &files, &sources, false);
        assert!(
            output.findings.is_empty() && output.advisories.is_empty(),
            "{rule}"
        );
        assert_eq!(sources.physical_read_count(), 0, "{rule}");
    }
}

#[test]
fn explicit_absolute_option_roots_share_the_discovered_path_identity() {
    let root = saved();
    for rule in [
        "rust-max-lines-per-file",
        "rust-no-inline-allows",
        "rust-no-inline-tests",
        "csharp-max-lines-per-file",
    ] {
        let mut config = config(&root, rule);
        let folder = config.rules[0].include[0].split('/').next().unwrap();
        let option_root = root.join(folder).join(".");
        config.rules[0].options["roots"] = serde_yaml::to_value([option_root]).unwrap();
        let files = files(&root, &config);
        let sources = super::super::source_store_for_files(&files);
        let output = evaluate(&root, &config, &files, &sources, false);
        assert!(!output.findings.is_empty(), "{rule}");
        assert_eq!(sources.physical_read_count(), files.len(), "{rule}");
    }
}

#[test]
fn requested_rust_fragments_report_unknown_and_exact_exclusions_do_not_read_them() {
    let root = saved();
    let selected = "rust-allows/expression-fragment.rs";
    for rule in ["rust-no-inline-allows", "rust-no-inline-tests"] {
        let mut config = config(&root, rule);
        config.rules[0].include = vec![selected.into()];
        let files = files(&root, &config);
        let sources = super::super::source_store_for_files(&files);
        let output = evaluate(&root, &config, &files, &sources, false);
        assert_eq!(output.findings.len(), 1, "{rule}");
        assert_eq!(output.findings[0].target.as_deref(), Some("unanalyzable"));
        assert_eq!(sources.physical_read_count(), 1);
        config.rules[0].exclude = vec![selected.into()];
        let excluded_sources = super::super::source_store_for_files(&files);
        let output = evaluate(&root, &config, &files, &excluded_sources, false);
        assert!(output.findings.is_empty() && output.advisories.is_empty());
        assert_eq!(excluded_sources.physical_read_count(), 0);
    }
}

mod suppression;

mod rust_work;

mod boundaries;
