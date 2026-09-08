use super::config::{self, Invocation, Options, Target, Traversal, UnknownCalls};
use super::*;
use crate::codebase::rules::run_check;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

fn options(yaml: &str) -> Options {
    serde_yaml::from_str(yaml).unwrap()
}

fn application(name: &str, yaml: &str) -> RuleDef {
    RuleDef {
        name: Some(name.to_string()),
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(yaml).unwrap(),
        ..Default::default()
    }
}

fn call_graph() -> (PathBuf, crate::codebase::dependencies::graph::DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/call-traversal/fixture"),
    );
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let graph = crate::codebase::dependencies::graph::DepGraph::build_with_plan(
        &root,
        &tsconfig,
        crate::codebase::dependencies::graph::GraphBuildPlan {
            calls: true,
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    (root, graph)
}

fn coverage_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/forbidden-calls/coverage/fixture"),
    )
}

fn coverage_findings(yaml: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    let root = coverage_root();
    let config = tempfile::Builder::new().suffix(".yml").tempfile()?;
    std::fs::write(config.path(), yaml)?;
    run_check(&root, Some(config.path()), None)
}

#[test]
fn validation_rejects_empty_and_invalid_selectors() {
    for yaml in [
        "targets: [{ global: setTimeout }]",
        "roots: [{ file: src/entry.mts }]",
        "roots: [{ file: src/entry.mts }]\nmaxDepth: 0\ntargets: [{ global: setTimeout }]",
        "roots: [{ vitest: false }]\ntargets: [{ global: setTimeout }]",
        "roots: [{ file: src/entry.mts }]\ntargets: [{ moduleExport: { module: '', export: setTimeout } }]",
        "roots: [{ file: src/entry.mts }]\ntargets: [{ function: { file: '', symbol: run } }]",
    ] {
        assert!(config::validate(&options(yaml)).is_err(), "{yaml}");
    }
    assert!(config::validate(&options(
        "roots: [{ file: src/entry.mts }]\nunknownCalls: finding"
    ))
    .is_ok());
}

#[test]
fn graph_plan_requests_calls_for_repeated_applications() {
    let config = NoMistakesConfig {
        rules: vec![
            application(
                "first",
                "roots: [{ file: src/entry.mts }]\ntargets: [{ exact: diamondLeft }]",
            ),
            application(
                "second",
                "roots: [{ file: src/entry.mts }]\ntargets: [{ terminal: diamondLeft }]",
            ),
        ],
        ..Default::default()
    };
    assert!(graph_plan(&config).is_some());
}

#[test]
fn applications_are_ordered_and_overlap_independently() {
    let (root, graph) = call_graph();
    let config = NoMistakesConfig {
        rules: vec![
            application("exact", "roots: [{ function: { file: src/entry.mts, symbol: entry } }]\ntargets: [{ exact: diamondLeft }]"),
            application("terminal", "roots: [{ function: { file: src/entry.mts, symbol: entry } }]\ntargets: [{ terminal: diamondLeft }]"),
        ],
        ..Default::default()
    };
    let files = vec![root.join("src/entry.mts")];
    let findings = check_with_graph(&root, &config, &graph, None, &files).unwrap();
    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert!(findings[0].message.contains("(exact, application #1)"));
    assert!(findings[1].message.contains("(terminal, application #2)"));
}

#[test]
fn traversal_and_invocation_options_deserialize_exactly() {
    let direct = options(
        "roots: [{ file: src/entry.mts }]\ntraversal: direct\ntargets: [{ exact: diamondLeft }]",
    );
    assert!(matches!(direct.traversal, Traversal::Direct));
    let file = options("roots: [{ module: src/entry.mts }]\ntraversal: file\nmaxDepth: 4\ninvocations: [construct]\ntargets: [{ terminal: Widget }]");
    assert!(matches!(file.traversal, Traversal::File));
    assert_eq!(file.max_depth, Some(4));
    assert!(matches!(
        file.invocations.as_slice(),
        [Invocation::Construct]
    ));
    let transitive =
        options("roots: [{ vitest: [unit] }]\ntraversal: transitive\nunknownCalls: finding");
    assert!(matches!(transitive.traversal, Traversal::Transitive));
    assert_eq!(transitive.unknown_calls, UnknownCalls::Finding);
}

#[test]
fn selector_variants_deserialize_without_textual_fallbacks() {
    let opts = options("roots: [{ file: src/entry.mts }]\ntargets:\n  - global: setTimeout\n  - exact: timers.setTimeout\n  - terminal: setTimeout\n  - moduleExport: { module: node:timers/promises, export: setTimeout }\n  - function: { file: src/timers.mts, symbol: sleep }");
    assert_eq!(opts.targets.len(), 5);
    assert!(matches!(opts.targets[0], Target::Global(_)));
    assert!(matches!(opts.targets[1], Target::Exact(_)));
    assert!(matches!(opts.targets[2], Target::Terminal(_)));
    assert!(matches!(opts.targets[3], Target::ModuleExport(_)));
    assert!(matches!(opts.targets[4], Target::Function(_)));
}

#[test]
fn aggregate_runner_keeps_distinct_same_line_occurrences() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/forbidden-calls/same-line-occurrences/fixture");
    let findings = run_check(&root, None, None).unwrap();
    let timer_findings = findings
        .iter()
        .filter(|finding| {
            finding.rule == RULE_ID
                && finding.file == "src/timers.mts"
                && finding.target.as_deref() == Some("global `setTimeout`")
        })
        .collect::<Vec<_>>();

    assert_eq!(timer_findings.len(), 2, "{findings:#?}");
    assert_eq!(timer_findings[0].line, timer_findings[1].line);
}

#[test]
fn traversal_limits_and_cycles_are_deterministic() {
    let direct = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ function: { file: src/entry.mts, symbol: entry } }]\n      traversal: direct\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();
    let transitive = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ function: { file: src/entry.mts, symbol: entry } }]\n      traversal: transitive\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();
    let capped = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ function: { file: src/entry.mts, symbol: entry } }]\n      traversal: transitive\n      maxDepth: 1\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();

    assert_eq!(direct.len(), 1, "{direct:#?}");
    assert_eq!(capped.len(), direct.len(), "{capped:#?}");
    assert_eq!(transitive.len(), 3, "{transitive:#?}");
}

#[test]
fn file_and_module_roots_select_their_callable_sources() {
    let findings = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/entry.mts }, { module: src/selectors.mts }]\n      traversal: file\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.file == "src/entry.mts"));
    assert!(findings
        .iter()
        .all(|finding| finding.file != "src/selectors.mts"));
}

#[test]
fn binding_aware_targets_distinguish_global_module_and_repository_calls() {
    for (target, expected) in [
        ("{ global: setTimeout }", 0),
        ("{ exact: timers.setTimeout }", 1),
        ("{ terminal: setTimeout }", 2),
        (
            "{ moduleExport: { module: node:timers, export: setTimeout } }",
            2,
        ),
        (
            "{ function: { file: src/targets.mts, symbol: repositoryTarget } }",
            2,
        ),
    ] {
        let findings = coverage_findings(&format!(
            "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{{ file: src/selectors.mts }}]\n      traversal: file\n      targets: [{target}]\n"
        ))
        .unwrap();
        assert_eq!(findings.len(), expected, "{target}: {findings:#?}");
    }
}

#[test]
fn construct_and_unknown_call_policies_are_explicit() {
    let construct = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/selectors.mts }]\n      traversal: file\n      invocations: [construct]\n      targets: [{ global: Date }]\n",
    )
    .unwrap();
    let ignored = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ function: { file: src/selectors.mts, symbol: unknownCall } }]\n      unknownCalls: ignore\n      targets: [{ global: neverCalled }]\n",
    )
    .unwrap();
    let reported = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ function: { file: src/selectors.mts, symbol: unknownCall } }]\n      unknownCalls: finding\n",
    )
    .unwrap();

    assert_eq!(construct.len(), 1, "{construct:#?}");
    assert!(ignored.is_empty(), "{ignored:#?}");
    assert_eq!(reported.len(), 1, "{reported:#?}");
}

#[test]
fn named_vitest_roots_use_the_prepared_catalog() {
    let findings = coverage_findings(
        "tests:\n  vitest:\n    configs: vitest.config.ts\nrules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ vitest: [unit] }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/unit/timer.test.mts");
}

#[test]
fn all_vitest_roots_and_default_call_invocations_are_inclusive() {
    let vitest = coverage_findings(
        "tests:\n  vitest:\n    configs: vitest.config.ts\nrules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ vitest: true }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();
    let default_calls = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/selectors.mts }]\n      traversal: file\n      targets: [{ global: Date }]\n",
    )
    .unwrap();

    assert_eq!(vitest.len(), 2, "{vitest:#?}");
    assert_eq!(default_calls.len(), 1, "{default_calls:#?}");
}

#[test]
fn malformed_selector_configuration_fails_through_the_public_runner() {
    let error = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/selectors.mts }]\n      targets: [{ moduleExport: bad }]\n",
    )
    .unwrap_err();

    assert!(error.to_string().contains("invalid options"), "{error:#}");
}

#[test]
fn application_path_filters_restrict_transitive_call_findings() {
    let findings = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    include: [src/entry.mts]\n    options:\n      roots: [{ file: src/entry.mts }]\n      traversal: transitive\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/entry.mts");
}

#[test]
fn ambiguous_selector_maps_are_rejected() {
    let error = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/entry.mts }]\n      targets: [{ global: setTimeout, exact: window.setTimeout }]\n",
    )
    .unwrap_err();

    assert!(error.to_string().contains("invalid options"), "{error:#}");
}

#[test]
fn repeated_applications_and_invalid_roots_stay_separate() {
    let findings = coverage_findings(
        "rules:\n  - name: same\n    rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/entry.mts }]\n      targets: [{ global: setTimeout }]\n  - name: same\n    rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/entry.mts }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();
    let error = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: missing.mts }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap_err();

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_ne!(findings[0].message, findings[1].message);
    assert!(error
        .to_string()
        .contains("file root `missing.mts` is not a file"));
}

#[test]
fn source_suppression_filters_call_findings_without_hiding_configuration_errors() {
    let suppressed = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      roots: [{ file: src/suppressed.mts }]\n      targets: [{ global: setTimeout }]\n",
    )
    .unwrap();
    let invalid = coverage_findings(
        "rules:\n  - rule: forbidden-calls\n    scope: repository\n    options:\n      targets: [{ global: setTimeout }]\n",
    );

    assert!(suppressed.is_empty(), "{suppressed:#?}");
    assert!(
        invalid.is_err(),
        "invalid configuration must not be suppressed"
    );
}
