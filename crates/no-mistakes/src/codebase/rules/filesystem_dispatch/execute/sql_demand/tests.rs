use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{collect_check_facts, CheckFactPlan};

fn repository() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
    )
}

fn outcome(
    root: &Path,
    config_path: &Path,
    consumer: &str,
    files: Vec<PathBuf>,
) -> Vec<crate::codebase::check_outcome::CheckFailure> {
    let config = crate::config::v2::load_v2_config(root, Some(config_path)).unwrap();
    let mut plan = CheckFactPlan {
        graph: crate::codebase::ts_source::facts::TsFactPlan::imports(),
        ..Default::default()
    };
    crate::codebase::postgres::assign_prepared_embedded_sql_options(&config, &mut plan).unwrap();
    crate::codebase::postgres::configure_prepared_postgres_plan(root, &config, &mut plan).unwrap();
    let facts = collect_check_facts(root, files.clone(), plan);
    assert!(facts.ts.values().any(|facts| facts.parse_error.is_some()));
    let session = AnalysisSession::disabled();
    session.enable_partial_checks();
    super::super::outcomes::register_demand(
        root,
        &config,
        consumer,
        &files,
        Some(&facts),
        &crate::codebase::rules::source_store_for_files(&files),
        &session,
    )
    .unwrap();
    session.check_failures()
}

#[test]
fn sql_only_and_fully_excluded_applications_require_no_typescript_sources() {
    let repo = repository();
    let root = repo.join("test-cases/rules/postgres-sql-statement-policy/fixture/embedded");
    let config = repo.join(
        "test-cases/rules/postgres-sql-statement-policy/configs/legacy_sql_only_skips_ts.yml",
    );
    assert!(outcome(
        &root,
        &config,
        "postgres-sql-statement-policy",
        vec![root.join("unparsed.ts")]
    )
    .is_empty());
    let root = repo.join("fixtures/postgres-facts/embedded");
    let config =
        repo.join("test-cases/rules/postgres-require-query-annotation/configs/excluded-calls.yml");
    assert!(outcome(
        &root,
        &config,
        "postgres-require-query-annotation",
        vec![
            root.join("undeclared-executor-types.ts"),
            root.join("helpers/uncooked-template.ts")
        ]
    )
    .is_empty());
}

#[test]
fn live_primary_keeps_its_failed_literal_helper_prerequisite() {
    let root = repository().join("fixtures/check/sql-helper-prerequisite");
    let failures = outcome(
        &root,
        &root.join(".no-mistakes.yml"),
        "postgres-require-query-annotation",
        vec![root.join("caller.ts"), root.join("helper.ts")],
    );
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].consumer, "postgres-require-query-annotation");
    assert_eq!(failures[0].file.as_ref(), Some(&root.join("helper.ts")));
}

#[test]
fn unrelated_and_unused_failed_helpers_do_not_poison_selected_primaries() {
    let root = repository().join("fixtures/check/sql-helper-prerequisite");
    for config in ["no-helper.yml", "unused-helper.yml"] {
        assert!(
            outcome(
                &root,
                &root.join(config),
                "postgres-require-query-annotation",
                vec![
                    root.join("no-helper.ts"),
                    root.join("unused-helper.ts"),
                    root.join("helper.ts")
                ]
            )
            .is_empty(),
            "{config}"
        );
    }
}

#[test]
fn repeated_unknown_helper_lookup_preserves_failed_source() {
    let root = repository().join("fixtures/check/sql-helper-prerequisite");
    let failures = outcome(
        &root,
        &root.join("repeated.yml"),
        "postgres-require-query-annotation",
        vec![root.join("repeated.ts"), root.join("helper.ts")],
    );
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].file.as_ref(), Some(&root.join("helper.ts")));
}

#[test]
fn demand_profiles_respect_test_helper_scopes_and_skip_non_sources() {
    let root = repository().join("fixtures/check/sql-helper-prerequisite");
    let candidates = ["caller.ts", "helper.ts", "empty.txt"].map(|name| root.join(name));
    for (consumer, effort, options, expected) in [
        (
            "postgres-no-test-state-mutation",
            "low",
            "testGlobs: [caller.ts]\nhelperRoots: [helper.ts]",
            vec!["caller.ts", "helper.ts"],
        ),
        (
            "postgres-no-offset",
            "low",
            "include: [caller.ts]",
            vec!["caller.ts"],
        ),
        (
            "postgres-require-query-annotation",
            "low",
            "include: [caller.ts]\nsqlLiteralHelpers: ['./helper#head']",
            vec!["caller.ts"],
        ),
        (
            "postgres-require-query-annotation",
            "max",
            "include: [helper.ts]",
            vec!["helper.ts"],
        ),
    ] {
        let mut config: NoMistakesConfig = serde_yaml::from_str(&format!(
            "effort: {effort}\nrules:\n  - rule: {consumer}\n    scope: repository\n"
        ))
        .unwrap();
        config.rules[0].options = serde_yaml::from_str(&format!(
            "importSpecifier: '@example/db'\nexecutorNames: [query]\n{options}"
        ))
        .unwrap();
        config.rules[0].resolved_application_index = Some(2);
        config.rules[0].resolved_analysis = Some(crate::config::v2::EffectiveRuleAnalysis {
            effort: config.effort.unwrap(),
            ..Default::default()
        });
        assert_eq!(
            super::files(&root, &config, consumer, &candidates, None).unwrap(),
            expected
                .iter()
                .map(|name| root.join(name))
                .collect::<Vec<_>>(),
            "{consumer}/{effort}"
        );
    }
}
