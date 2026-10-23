use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::config::v2::schema::{RuleDef, RuleScope};
use crate::config::v2::{EffectiveRuleAnalysis, Effort, NoMistakesConfig, Unanalyzable};
use std::sync::Arc;

const WORKFLOW: &str = ".github/workflows/ci.yml";

fn fixture_root(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/github-actions-download-integrity/fixture")
            .join(name),
    )
}

fn source(name: &str) -> String {
    std::fs::read_to_string(fixture_root(name).join(WORKFLOW)).unwrap()
}

fn hosts() -> Vec<String> {
    DEFAULT_TRUSTED_HOSTS
        .iter()
        .map(|host| (*host).to_string())
        .collect()
}

fn rule(effort: Effort, unanalyzable: Unanalyzable) -> RuleDef {
    RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        resolved_analysis: Some(EffectiveRuleAnalysis {
            effort,
            unanalyzable,
        }),
        ..RuleDef::default()
    }
}

fn argv_script(name: &str) -> String {
    let workflow: serde_yaml::Value = serde_yaml::from_str(&source("effort-argv")).unwrap();
    workflow["jobs"]["verify"]["steps"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|step| step["name"].as_str() == Some(name))
        .unwrap()["run"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn low_bypasses_assignment_expansion_but_max_still_recovers_the_url() {
    let workflow: serde_yaml::Value = serde_yaml::from_str(&source("fail")).unwrap();
    let script = workflow["jobs"]["build"]["steps"][2]["run"]
        .as_str()
        .unwrap();
    let before = assign::expansion_count();
    let binding_reads = assign::binding_scan_count();
    let low = judge::download_hits_low(script, &[]);
    assert_eq!(assign::expansion_count(), before);
    assert_eq!(assign::binding_scan_count(), binding_reads);
    assert_eq!(low.len(), 1);
    assert!(low[0].unanalyzable);
    assert_eq!(low[0].url, "$BASE_URL/b.tgz");

    let max = judge::download_hits(script, &[]);
    assert_eq!(assign::expansion_count(), before + 1);
    assert_eq!(assign::binding_scan_count(), binding_reads + 1);
    assert_eq!(max.len(), 1);
    assert!(!max[0].unanalyzable);
    assert_eq!(max[0].url, "https://example.invalid/b.tgz");
}

#[test]
fn low_routes_expansions_and_dynamic_paths_without_reclassifying_literal_findings() {
    let source = source("fail");
    let max = runs::check_source(WORKFLOW, &source, &hosts(), &[]);
    let report = runs::check_source_low(WORKFLOW, &source, &hosts(), &[], Unanalyzable::Report);
    let shapes: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.message.contains("unanalyzable at effort low"))
        .cloned()
        .collect();
    assert_eq!(shapes.len(), 5, "{:?}", report.findings);
    assert_eq!(report.findings.len(), max.len());
    let literal: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| !shapes.contains(finding))
        .cloned()
        .collect();
    for finding in &literal {
        assert!(max.contains(finding), "literal max parity: {finding:?}");
    }
    for policy in [Unanalyzable::Advise, Unanalyzable::Ignore] {
        let result = runs::check_source_low(WORKFLOW, &source, &hosts(), &[], policy);
        assert_eq!(result.findings, literal);
        if policy == Unanalyzable::Advise {
            assert_eq!(result.advisories, shapes);
        } else {
            assert!(result.advisories.is_empty());
        }
    }
}

#[test]
fn low_keeps_literal_trust_checksum_verification_and_skip_proofs() {
    let source = source("pass");
    assert!(runs::check_source(WORKFLOW, &source, &hosts(), &[]).is_empty());
    let low = runs::check_source_low(WORKFLOW, &source, &hosts(), &[], Unanalyzable::Advise);
    assert!(low.findings.is_empty(), "{:?}", low.findings);
    // Max resolves this trusted-host assignment; low reports only the unknown URL shape.
    assert_eq!(low.advisories.len(), 1, "{:?}", low.advisories);
    assert!(low.advisories[0]
        .message
        .contains("$BASE_URL/repos/example/tool"));
    assert!(low.advisories[0]
        .message
        .contains("inline a literal HTTP(S) URL"));
}

#[test]
fn applications_keep_independent_max_report_and_advise_policies() {
    let root = fixture_root("fail");
    let files = vec![root.join(WORKFLOW)];
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
    let config = NoMistakesConfig {
        rules: vec![
            rule(Effort::Low, Unanalyzable::Report),
            rule(Effort::Low, Unanalyzable::Advise),
            rule(Effort::Max, Unanalyzable::Ignore),
        ],
        ..NoMistakesConfig::default()
    };
    let result =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(result.advisories.len(), 5);
    assert_eq!(
        result
            .findings
            .iter()
            .filter(|finding| finding.message.contains("unanalyzable at effort low"))
            .count(),
        5
    );
    assert!(result.findings.iter().any(|finding| finding
        .message
        .contains("downloads `https://example.invalid/b.tgz`")));
}

#[test]
fn shared_analysis_options_are_sanitized_but_unknown_rule_options_still_fail() {
    let root = fixture_root("fail");
    let files = vec![root.join(WORKFLOW)];
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
    let mut application = rule(Effort::Low, Unanalyzable::Advise);
    application.options = serde_yaml::from_str("effort: low\nunanalyzable: advise\n").unwrap();
    let mut config = NoMistakesConfig {
        rules: vec![application],
        ..NoMistakesConfig::default()
    };
    let result =
        diagnostics_with_files_and_sources(&root, &config, &files, &sources, true).unwrap();
    assert_eq!(result.advisories.len(), 5);
    config.rules[0].options = serde_yaml::from_str("unexpected: true\n").unwrap();
    let error = diagnostics_with_files_and_sources(&root, &config, &files, &sources, true)
        .err()
        .unwrap();
    assert!(error.to_string().contains("invalid options"), "{error}");
}

#[test]
fn low_proves_downloader_alias_scope_without_expanding_url_arguments() {
    let helpers = ["ci_download_to".to_string()];
    for name in [
        "literal-assignment-command",
        "prefixed-command",
        "configured-helper-command",
        "literal-command-separators",
    ] {
        let script = argv_script(name);
        let before = assign::expansion_count();
        let binding_reads = assign::binding_scan_count();
        let low = judge::download_hits_low(&script, &helpers);
        assert_eq!(assign::expansion_count(), before, "{name}");
        assert_eq!(assign::binding_scan_count(), binding_reads + 1, "{name}");
        assert_eq!(low.len(), 1, "{name}");
        assert!(low[0].unanalyzable && low[0].indirect_command, "{name}");
        let max = judge::download_hits(&script, &helpers);
        assert_eq!(max.len(), 1, "{name}");
        assert!(!max[0].unanalyzable, "{name}");
    }
    assert!(judge::download_hits_low(&argv_script("non-downloader-command"), &helpers).is_empty());
}

#[test]
fn low_does_not_claim_unsafe_urls_when_dynamic_arguments_can_change_url_slots() {
    for name in [
        "dynamic-output-flags",
        "static-host-dynamic-path",
        "mixed-literal-and-dynamic-url-command",
    ] {
        let low = judge::download_hits_low(&argv_script(name), &[]);
        assert_eq!(low.len(), 1, "{name}");
        assert!(low.iter().all(|hit| hit.unanalyzable), "{name}");
    }
    // At max this argument becomes --output, making the URL an output filename.
    assert!(judge::download_hits(&argv_script("dynamic-output-flags"), &[]).is_empty());
}

#[test]
fn low_keeps_non_url_option_operands_out_of_url_diagnostics() {
    for name in [
        "dynamic-output-operand",
        "dynamic-header-operand",
        "dynamic-wget-output-operand",
    ] {
        let script = argv_script(name);
        let low = judge::filter_trusted(judge::download_hits_low(&script, &[]), &hosts());
        assert!(low.is_empty(), "{name}");
    }
}

#[test]
fn non_ascii_non_url_operands_do_not_panic_during_prefix_classification() {
    let script = argv_script("unicode-non-url-operand");
    assert!(judge::download_hits(&script, &[]).is_empty());
    assert!(judge::download_hits_low(&script, &[]).is_empty());
}
