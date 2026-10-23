use super::*;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};
use crate::codebase::rules::analysis_policy::prepare_rule_analysis;
use crate::config::v2::{Effort, Unanalyzable};

fn effort_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/no-nextjs-caching-effort/fixture")
}

fn prepared(
    effort: Option<Effort>,
    policy: Unanalyzable,
) -> (PathBuf, NoMistakesConfig, CheckFactMap) {
    let root = effort_fixture();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let config_name = if effort == Some(Effort::Low) {
        ".no-mistakes.low.yml"
    } else {
        ".no-mistakes.max.yml"
    };
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(config_name))).unwrap();
    config.effort_override = effort;
    config.unanalyzable = Some(policy);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
    let mut plan = CheckFactPlan {
        source: true,
        ..Default::default()
    };
    configure_effort_plan(&config, &mut plan).unwrap();
    let facts = collect_check_facts(&root, files, plan);
    (root, config, facts)
}

fn run(
    effort: Option<Effort>,
    policy: Unanalyzable,
) -> super::super::super::analysis_policy::AnalysisDiagnostics {
    let (root, config, facts) = prepared(effort, policy);
    diagnostics_with_facts_for_aggregate(&root, &config, &facts, None, false).unwrap()
}

#[test]
fn low_skips_max_extraction_and_max_retains_default_output() {
    let (_, _, facts) = prepared(Some(Effort::Low), Unanalyzable::Advise);
    assert!(facts
        .ts
        .values()
        .all(|facts| facts.nextjs_caching.is_none()));
    assert!(facts
        .ts
        .values()
        .all(|facts| facts.nextjs_caching_low.is_some()));
    let default = run(None, Unanalyzable::Report);
    let max = run(Some(Effort::Max), Unanalyzable::Report);
    assert_eq!(max.findings, default.findings);
    assert_eq!(max.findings.len(), 11);
    assert!(max.advisories.is_empty());
}

#[test]
fn low_literal_findings_are_a_subset_of_max_and_residuals_follow_policy() {
    for policy in [Unanalyzable::Ignore, Unanalyzable::Advise] {
        let low = run(Some(Effort::Low), policy);
        let max = run(Some(Effort::Max), policy);
        assert!(!low.findings.is_empty());
        assert!(low.findings.len() < max.findings.len());
        assert!(low
            .findings
            .iter()
            .all(|finding| max.findings.contains(finding)));
        assert_eq!(
            low.advisories.len(),
            if policy == Unanalyzable::Advise { 4 } else { 0 }
        );
        assert!(low
            .advisories
            .iter()
            .all(|finding| finding.message.contains("literal fetch options")));
        assert!(!low.advisories.iter().any(|finding| finding.line == 12));
    }
    let report = run(Some(Effort::Low), Unanalyzable::Report);
    let advise = run(Some(Effort::Low), Unanalyzable::Advise);
    assert!(report.advisories.is_empty());
    assert!(run(Some(Effort::Max), Unanalyzable::Report).findings.len() >= report.findings.len());
    assert_eq!(
        report.findings.len(),
        advise.findings.len() + advise.advisories.len()
    );
}

#[test]
fn mixed_applications_request_both_projections_and_keep_max_application() {
    let (root, mut config, _) = prepared(Some(Effort::Low), Unanalyzable::Advise);
    config.effort_override = None;
    let mut max = config.rules[0].clone();
    max.options = serde_yaml::from_str("{ effort: max }").unwrap();
    config.rules[0].options =
        serde_yaml::from_str("{ effort: low, unanalyzable: advise }").unwrap();
    config.rules.push(max);
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
    let mut plan = CheckFactPlan {
        source: true,
        ..Default::default()
    };
    configure_effort_plan(&config, &mut plan).unwrap();
    assert!(plan.nextjs_caching && plan.nextjs_caching_low);
    let facts = collect_check_facts(&root, files, plan);
    let mixed = diagnostics_with_facts_for_aggregate(&root, &config, &facts, None, false).unwrap();
    let max = run(Some(Effort::Max), Unanalyzable::Advise);
    assert!(max
        .findings
        .iter()
        .all(|finding| mixed.findings.contains(finding)));
    assert_eq!(mixed.advisories.len(), 4);
}

#[test]
fn standalone_and_prepared_low_findings_match() {
    let (root, config, facts) = prepared(Some(Effort::Low), Unanalyzable::Report);
    assert_eq!(
        check(&root, &config).unwrap(),
        diagnostics_with_facts_for_aggregate(&root, &config, &facts, None, false)
            .unwrap()
            .findings
    );
}

#[test]
fn low_saved_shapes_route_opaque_inputs_and_preserve_direct_checks() {
    let root = effort_fixture().parent().unwrap().join("shape-fixture");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = collect_check_facts(
        &root,
        files.clone(),
        CheckFactPlan {
            nextjs_caching_low: true,
            source: true,
            ..Default::default()
        },
    );
    let mut config = config();
    config.effort = Some(Effort::Low);
    config.unanalyzable = Some(Unanalyzable::Advise);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
    let diagnostics =
        diagnostics_with_facts_for_aggregate(&root, &config, &facts, None, false).unwrap();
    assert!(diagnostics
        .findings
        .iter()
        .any(|finding| finding.message.contains("cache directives")));
    assert!(diagnostics
        .findings
        .iter()
        .any(|finding| finding.message.contains("next.tags")));
    assert!(!diagnostics
        .findings
        .iter()
        .chain(&diagnostics.advisories)
        .any(|finding| finding.file.ends_with("shadow.ts") || finding.file.ends_with("local.ts")));
    assert!(diagnostics
        .advisories
        .iter()
        .any(|finding| finding.file.ends_with("route.ts")));
    assert!(diagnostics
        .advisories
        .iter()
        .any(|finding| finding.file.ends_with("next.config.cjs")));
    // Spreads can supply fetch options; destructured segment names stay opaque.
    for (file, count) in [("spread.ts", 2), ("app/nested/layout.ts", 3)] {
        assert_eq!(
            diagnostics
                .advisories
                .iter()
                .filter(|finding| finding.file.ends_with(file))
                .count(),
            count
        );
        assert!(!diagnostics
            .findings
            .iter()
            .any(|finding| finding.file.ends_with(file)));
    }
    assert!(diagnostics
        .advisories
        .iter()
        .any(|finding| finding.file.ends_with("app/route.ts") && finding.line == 4));
    for file in &files {
        let source = std::fs::read_to_string(file).unwrap();
        let direct = crate::ast::with_program(file, &source, |program, _| {
            low::extract_program(file, &source, program)
        })
        .unwrap();
        assert_eq!(
            facts
                .ts
                .get(file)
                .unwrap()
                .nextjs_caching_low
                .as_ref()
                .unwrap(),
            &direct
        );
    }
}

#[test]
fn effort_planning_rejects_invalid_policy_before_requesting_facts() {
    let mut config = config();
    config.rules[0].options = serde_yaml::from_str("{ effort: invalid }").unwrap();
    let mut plan = CheckFactPlan::default();
    let error = configure_effort_plan(&config, &mut plan).unwrap_err();
    assert!(error.to_string().contains("invalid options.effort"));
    assert!(!plan.nextjs_caching && !plan.nextjs_caching_low);
}
