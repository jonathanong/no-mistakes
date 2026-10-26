use super::config::{self, Invocation, Options, Target, Traversal, UnknownCalls};
use super::*;
use crate::codebase::rules::run_check;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;

mod support;
pub(super) use support::{expand_roots, finding_for_site};

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

fn forbidden_calls_case(name: &str) -> PathBuf {
    crate::test_support::rule_fixture_root("forbidden-calls").join(format!("cases/{name}.yml"))
}

fn coverage_findings(case: &str) -> anyhow::Result<Vec<crate::codebase::rules::RuleFinding>> {
    run_check(&coverage_root(), Some(&forbidden_calls_case(case)), None)
}

mod config_validation;

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
    let findings = check_with_graph(&root, &config, &graph, None, None, &files, None).unwrap();
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
fn function_and_terminal_selectors_use_canonical_repository_targets() {
    use crate::codebase::dependencies::extract::InvocationKind;
    use crate::codebase::dependencies::graph::{ResolvedCallSite, ResolvedCallTarget};

    let root = coverage_root();
    let app = application(
        "canonical",
        "roots: [{ file: src/selectors.mts }]\ntargets: [{ function: { file: src/targets.mts, symbol: repositoryTarget } }]",
    );
    let opts = options(
        "roots: [{ file: src/selectors.mts }]\ntargets: [{ function: { file: src/targets.mts, symbol: repositoryTarget } }]",
    );
    let repository_site = ResolvedCallSite {
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
        file: root.join("src/selectors.mts"),
        caller: Some("selectorCalls".to_string()),
        caller_id: None,
        line: 8,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: "repositoryTarget".to_string(),
        target: ResolvedCallTarget::RepositoryFunction {
            file: root.join("src/targets.mts"),
            scope: "repositoryTarget".to_string(),
        },
    };
    assert_eq!(
        finding_for_site(
            &root,
            &app,
            "canonical, application #1",
            &opts,
            &repository_site
        )
        .expect("canonical repository target matches")
        .target
        .as_deref(),
        Some("repository function `src/targets.mts#repositoryTarget`")
    );

    let terminal =
        options("roots: [{ file: src/selectors.mts }]\ntargets: [{ terminal: repositoryTarget }]");
    assert_eq!(
        finding_for_site(
            &root,
            &app,
            "canonical, application #1",
            &terminal,
            &repository_site
        )
        .expect("terminal selector matches repository scope")
        .target
        .as_deref(),
        Some("terminal `repositoryTarget`")
    );
}

#[test]
fn vitest_roots_require_a_catalog_and_absolute_function_roots_resolve() {
    let (root, graph) = call_graph();
    let files = vec![root.join("src/entry.mts")];
    let vitest = options("roots: [{ vitest: true }]\ntargets: [{ global: setTimeout }]");
    let error = expand_roots(&root, &vitest, &graph, &files).unwrap_err();
    assert!(error
        .to_string()
        .contains("require a prepared Vitest project catalog"));

    let entry = root.join("src/entry.mts");
    let function = options(&format!(
        "roots: [{{ function: {{ file: {}, symbol: entry }} }}]\ntargets: [{{ global: setTimeout }}]",
        entry.display()
    ));
    let nodes = expand_roots(&root, &function, &graph, &files)
        .expect("absolute function roots resolve from the configured file");
    assert!(!nodes.is_empty());
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
    let direct = coverage_findings("traversal-direct").unwrap();
    let transitive = coverage_findings("traversal-transitive").unwrap();
    let capped = coverage_findings("traversal-max-depth-1").unwrap();

    assert_eq!(direct.len(), 1, "{direct:#?}");
    assert_eq!(capped.len(), direct.len(), "{capped:#?}");
    assert_eq!(transitive.len(), 3, "{transitive:#?}");
}

#[test]
fn file_and_module_roots_select_their_callable_sources() {
    let findings = coverage_findings("file-and-module-roots").unwrap();

    assert!(findings
        .iter()
        .any(|finding| finding.file == "src/entry.mts"));
    assert!(findings
        .iter()
        .all(|finding| finding.file != "src/selectors.mts"));

    let error = coverage_findings("module-without-callable-source")
        .expect_err("a module without callable sources must be rejected");
    assert!(
        error
            .to_string()
            .contains("configured root resolves to no callable source"),
        "{error:#}"
    );
}

#[test]
fn binding_aware_targets_distinguish_global_module_and_repository_calls() {
    for (case, target, expected) in [
        ("selectors-global-set-timeout", "{ global: setTimeout }", 0),
        (
            "selectors-exact-timers-set-timeout",
            "{ exact: timers.setTimeout }",
            1,
        ),
        (
            "selectors-terminal-set-timeout",
            "{ terminal: setTimeout }",
            2,
        ),
        (
            "selectors-module-export-set-timeout",
            "{ moduleExport: { module: node:timers, export: setTimeout } }",
            2,
        ),
        (
            "selectors-function-repository-target",
            "{ function: { file: src/targets.mts, symbol: repositoryTarget } }",
            2,
        ),
    ] {
        let findings = coverage_findings(case).unwrap();
        assert_eq!(findings.len(), expected, "{target}: {findings:#?}");
    }
}

#[test]
fn construct_and_unknown_call_policies_are_explicit() {
    let call = coverage_findings("invocations-call-date").unwrap();
    let construct = coverage_findings("invocations-construct-date").unwrap();
    let ignored = coverage_findings("unknown-calls-ignore").unwrap();
    let reported = coverage_findings("unknown-calls-finding").unwrap();

    assert_eq!(call.len(), 1, "{call:#?}");
    assert_eq!(construct.len(), 1, "{construct:#?}");
    assert!(ignored.is_empty(), "{ignored:#?}");
    assert_eq!(reported.len(), 1, "{reported:#?}");
}

#[test]
fn exact_selectors_match_unknown_member_calls_before_ignore() {
    let findings = coverage_findings("exact-page-wait-for-timeout").unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert!(
        findings[0].message.contains("exact `page.waitForTimeout`"),
        "{findings:#?}"
    );
}

#[test]
fn named_vitest_roots_use_the_prepared_catalog() {
    let findings = coverage_findings("vitest-named-unit").unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/unit/timer.test.mts");
}

#[test]
fn all_vitest_roots_and_default_call_invocations_are_inclusive() {
    let vitest = coverage_findings("vitest-all-roots").unwrap();
    let default_calls = coverage_findings("default-date-calls").unwrap();

    assert_eq!(vitest.len(), 2, "{vitest:#?}");
    assert_eq!(default_calls.len(), 1, "{default_calls:#?}");
}

#[test]
fn prepared_vitest_catalog_handles_unconfigured_and_unknown_projects() {
    let root = coverage_root();
    let visible = crate::codebase::ts_source::discover_files(&root, &[]);
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(&root, &visible);
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths_dir: root.clone(),
        ..Default::default()
    };
    let tsconfig_catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(&root, tsconfig, None);

    let unconfigured: NoMistakesConfig = serde_yaml::from_str(
        "tests:\n  vitest:\n    configs: missing.config.ts\n    projects:\n      unit:\n        include: [src/**/*.test.mts]",
    )
    .unwrap();
    let empty = crate::codebase::rules::prepare_vitest_project_catalog(
        &root,
        &unconfigured,
        &snapshot,
        &tsconfig_catalog,
    );
    assert!(empty.config_projects().unwrap().is_empty());

    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let catalog = crate::codebase::rules::prepare_vitest_project_catalog(
        &root,
        &config,
        &snapshot,
        &tsconfig_catalog,
    );
    let error = catalog
        .matching_files(&root, &["missing".to_string()], &visible)
        .unwrap_err();
    assert!(error.to_string().contains("names an unknown project"));
}

#[test]
fn malformed_selector_configuration_fails_through_the_public_runner() {
    let error = coverage_findings("malformed-module-export").unwrap_err();

    assert!(error.to_string().contains("invalid options"), "{error:#}");
}

#[test]
fn application_path_filters_restrict_transitive_call_findings() {
    let findings = coverage_findings("include-restricts-transitive").unwrap();

    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/entry.mts");
}

#[test]
fn ambiguous_selector_maps_are_rejected() {
    let error = coverage_findings("ambiguous-selector-map").unwrap_err();

    assert!(error.to_string().contains("invalid options"), "{error:#}");
}

#[test]
fn repeated_applications_and_invalid_roots_stay_separate() {
    let findings = coverage_findings("repeated-applications").unwrap();
    let error = coverage_findings("missing-file-root").unwrap_err();

    assert_eq!(findings.len(), 2, "{findings:#?}");
    assert_ne!(findings[0].message, findings[1].message);
    assert!(error
        .to_string()
        .contains("file root `missing.mts` is not a file"));
}

#[test]
fn source_suppression_filters_call_findings_without_hiding_configuration_errors() {
    let suppressed = coverage_findings("suppressed-source").unwrap();
    let invalid = coverage_findings("missing-roots");

    assert!(suppressed.is_empty(), "{suppressed:#?}");
    assert!(
        invalid.is_err(),
        "invalid configuration must not be suppressed"
    );
}

mod explicit_playwright;
mod explicit_vitest;
mod file_bounded;
mod glob;
mod parse_errors;
mod path_memo;
mod playwright;
mod terminal;
