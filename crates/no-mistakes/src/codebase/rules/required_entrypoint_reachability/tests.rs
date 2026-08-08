use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/required-entrypoint-reachability/fixture"),
    )
}

fn application(options: &str) -> RuleDef {
    RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    }
}

fn run(mut rules: Vec<RuleDef>) -> Vec<RuleFinding> {
    let root = fixture();
    let files = crate::codebase::dependencies::graph::GraphFiles::discover(&root)
        .all()
        .to_vec();
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible(
        Some(&root.join("tsconfig.json")),
        &root,
        &files,
    )
    .unwrap();
    let graph_files = crate::codebase::dependencies::graph::GraphFiles::from_files(files.clone());
    let graph = DepGraph::build_with_plan_and_files(
        &root,
        &tsconfig,
        GraphBuildPlan::imports_and_workspace(),
        &graph_files,
    )
    .unwrap();
    let mut config = NoMistakesConfig::default();
    config.rules.append(&mut rules);
    check_with_graph(&root, &config, &files, &graph).unwrap()
}

#[test]
fn accepts_static_dynamic_require_named_and_star_reexports() {
    let findings = run(vec![application(
        r#"
sourceGlobs:
  - sources/static.ts
  - sources/dynamic.ts
  - sources/required.ts
  - sources/named.ts
  - sources/star.ts
entrypoints: [entrypoints/api.ts]
"#,
    )]);

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn rejects_type_only_and_unreachable_sources_in_deterministic_order() {
    let findings = run(vec![application(
        r#"
sourceGlobs: [sources/unreachable.ts, sources/type-only.ts]
entrypoints: [entrypoints/api.ts]
"#,
    )]);

    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["sources/type-only.ts", "sources/unreachable.ts"]
    );
    assert!(findings
        .iter()
        .all(|finding| finding.message.contains("not runtime-reachable")));
}

#[test]
fn max_depth_is_applied_per_rule_application() {
    let pass = run(vec![application(
        r#"
sourceGlobs: [sources/named.ts]
entrypoints: [entrypoints/api.ts]
maxDepth: 2
"#,
    )]);
    let fail = run(vec![application(
        r#"
sourceGlobs: [sources/named.ts]
entrypoints: [entrypoints/api.ts]
maxDepth: 1
"#,
    )]);

    assert!(pass.is_empty(), "unexpected findings: {pass:?}");
    assert_eq!(fail.len(), 1);
    assert_eq!(fail[0].file, "sources/named.ts");
}

#[test]
fn rejects_missing_entrypoints_and_each_zero_match_source_pattern() {
    let findings = run(vec![application(
        r#"
sourceGlobs:
  - sources/static.ts
  - sources/no-match-b.ts
  - sources/no-match-a.ts
entrypoints: [entrypoints/missing.ts]
"#,
    )]);

    assert_eq!(findings.len(), 3);
    assert_eq!(
        findings.iter().map(|finding| finding.message.as_str()).collect::<Vec<_>>(),
        [
            "required-entrypoint-reachability: entrypoint `entrypoints/missing.ts` does not exist",
            "required-entrypoint-reachability: sourceGlobs pattern `sources/no-match-a.ts` matched no files",
            "required-entrypoint-reachability: sourceGlobs pattern `sources/no-match-b.ts` matched no files",
        ]
    );
}

#[test]
fn rejects_incomplete_options_and_invalid_source_globs() {
    let findings = run(vec![
        application("{}"),
        application(
            r#"
sourceGlobs: ["["]
entrypoints: [entrypoints/api.ts]
"#,
        ),
    ]);

    assert_eq!(findings.len(), 3);
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("requires at least one entrypoint")));
    assert!(findings.iter().any(|finding| finding
        .message
        .contains("requires at least one sourceGlobs pattern")));
    assert!(findings
        .iter()
        .any(|finding| finding.message.contains("invalid sourceGlobs pattern `[`")));
}

#[test]
fn common_exclude_removes_intentional_sources() {
    let mut rule = application(
        r#"
sourceGlobs: [sources/*.ts]
entrypoints: [entrypoints/api.ts]
"#,
    );
    rule.exclude = vec![
        "sources/suppressed.ts".to_string(),
        "sources/type-only.ts".to_string(),
        "sources/unreachable.ts".to_string(),
    ];

    let findings = run(vec![rule]);

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn repeated_source_globs_are_checked_against_each_applications_entrypoints() {
    let findings = run(vec![
        application(
            r#"
sourceGlobs: [sources/static.ts, sources/dynamic.ts]
entrypoints: [entrypoints/api.ts]
"#,
        ),
        application(
            r#"
sourceGlobs: [sources/static.ts, sources/dynamic.ts]
entrypoints: [entrypoints/worker.ts]
"#,
        ),
    ]);

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].file, "sources/dynamic.ts");
    assert_eq!(findings[0].target.as_deref(), Some("entrypoints/worker.ts"));
}

#[test]
fn aggregate_runner_honors_file_suppression() {
    let root = fixture();
    let findings = crate::codebase::rules::run_check(
        &root,
        Some(&root.join("suppression.no-mistakes.yml")),
        Some(&root.join("tsconfig.json")),
    )
    .unwrap();

    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}
