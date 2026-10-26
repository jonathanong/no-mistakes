use super::*;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources, CheckFactPlan,
};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::RuleFinding;

fn run(config: &mut NoMistakesConfig) -> (AnalysisDiagnostics, usize) {
    let root = fixture();
    let path = root.join("src/adopter.ts");
    let files = vec![path.clone()];
    let sources = super::super::super::source_store_for_files(&files);
    prepare_rule_analysis_with_support(&root, config, &files, &sources, ADOPTERS).unwrap();
    let policy = config.effective_rule_analysis(&config.rules[0]).unwrap();
    let plan = CheckFactPlan {
        graph: crate::codebase::ts_source::facts::TsFactPlan {
            function_calls: true,
            value_flow: policy.effort == Effort::Max,
            ..Default::default()
        },
        ..Default::default()
    };
    let facts = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        files,
        Vec::new(),
        plan,
        None,
        sources.clone(),
    );
    let ts = facts.get_ts_facts(&path).unwrap();
    assert_eq!(facts.stats.files_parsed, 1);
    assert_eq!(sources.physical_read_count(), 1);
    let mut diagnostics = AnalysisDiagnostics::default();
    for call in ts
        .function_calls
        .iter()
        .filter(|call| call.callee == "inspect")
    {
        let message = match call.static_arg.as_deref() {
            Some("allowed") => continue,
            Some(_) => "the literal is forbidden",
            None => "pass a literal directly instead of calling a helper",
        };
        let finding = RuleFinding {
            rule: "test-effort-adopter".into(),
            file: "src/adopter.ts".into(),
            line: call.line as usize,
            message: message.into(),
            import: None,
            target: None,
        };
        if call.static_arg.is_some() {
            diagnostics.findings.push(finding);
        } else {
            diagnostics.unanalyzable(policy.unanalyzable, finding);
        }
    }
    (diagnostics, ts.value_flow.calls.len())
}

#[test]
fn real_test_adopter_judges_literals_and_routes_unsupported_shapes() {
    for policy in [
        Unanalyzable::Report,
        Unanalyzable::Advise,
        Unanalyzable::Ignore,
    ] {
        let mut config = prepare("covered.yml", ADOPTERS);
        // The saved consumer is the complete scope for this invocation.
        config.rules[0].include = vec!["src/adopter.ts".into()];
        config.rules[1].include = config.rules[0].include.clone();
        config.rules[0].options["unanalyzable"] = serde_yaml::to_value(policy).unwrap();
        let (diagnostics, deep_calls) = run(&mut config);
        assert_eq!(
            deep_calls, 0,
            "low must omit value-flow before the fact pass"
        );
        assert_eq!(diagnostics.findings[0].line, 4);
        assert_eq!(diagnostics.findings[0].message, "the literal is forbidden");
        assert!(diagnostics.has_failures(), "supported shapes remain judged");
        assert_eq!(
            diagnostics.findings.len(),
            1 + usize::from(policy == Unanalyzable::Report)
        );
        assert_eq!(
            diagnostics.advisories.len(),
            usize::from(policy == Unanalyzable::Advise)
        );
        for finding in diagnostics
            .findings
            .iter()
            .chain(&diagnostics.advisories)
            .filter(|finding| finding.line == 5)
        {
            assert!(finding.message.contains("pass a literal directly"));
        }
    }
    let mut config = prepare("missing.yml", ADOPTERS);
    let (_, deep_calls) = run(&mut config);
    assert!(
        deep_calls >= 3,
        "partner fallback must request max fact demand"
    );
    assert_eq!(config.analysis_warnings.len(), 1);
}

#[test]
fn unsupported_only_diagnostics_advise_without_failing_and_ignore_without_output() {
    // Use the actual unsupported-site diagnostic from the prepared saved rule.
    let mut config = prepare("covered.yml", ADOPTERS);
    config.rules[0].include = vec!["src/adopter.ts".into()];
    config.rules[1].include = config.rules[0].include.clone();
    let (diagnostics, _) = run(&mut config);
    let residual = diagnostics.advisories.into_iter().next().unwrap();
    for (policy, advisory_count) in [(Unanalyzable::Advise, 1), (Unanalyzable::Ignore, 0)] {
        let mut output = AnalysisDiagnostics::default();
        output.unanalyzable(policy, residual.clone());
        assert!(!output.has_failures());
        assert_eq!(output.advisories.len(), advisory_count);
    }
}
