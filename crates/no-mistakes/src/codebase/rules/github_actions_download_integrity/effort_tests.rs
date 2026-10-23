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

#[test]
fn low_bypasses_assignment_expansion_but_max_still_recovers_the_url() {
    let workflow: serde_yaml::Value = serde_yaml::from_str(&source("fail")).unwrap();
    let script = workflow["jobs"]["build"]["steps"][2]["run"]
        .as_str()
        .unwrap();
    let before = assign::probe::count();
    let binding_reads = assign::probe::binding_scans();
    let low = judge::download_hits_low(script, &[]);
    assert_eq!(assign::probe::count(), before);
    assert_eq!(assign::probe::binding_scans(), binding_reads);
    assert_eq!(low.len(), 1);
    assert!(low[0].unanalyzable);
    assert_eq!(low[0].url, "$BASE_URL/b.tgz");

    let max = judge::download_hits(script, &[]);
    assert_eq!(assign::probe::count(), before + 1);
    assert_eq!(assign::probe::binding_scans(), binding_reads + 1);
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

mod argv;
