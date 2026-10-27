use super::*;

mod prefix_policy;
use crate::config::v2::schema::{RuleDef, RuleScope};
use std::sync::Arc;

// Output cases moved to cli_postgres_snapshot_no_test_state_mutation.
// Verdict ledger fields stay here: check --format json omits them.

#[test]
fn only_requested_database_mutations_match() {
    for kind in [
        "CREATE FUNCTION",
        "ALTER TYPE",
        "DROP MATERIALIZED VIEW",
        "ALTER SYSTEM",
        "TRUNCATE",
    ] {
        assert!(super::scan::banned_kind(kind), "{kind}");
    }
    for kind in ["INSERT", "UPDATE", "COMMENT", "CREATE ROLE", "DROP POLICY"] {
        assert!(!super::scan::banned_kind(kind), "{kind}");
    }
}

#[test]
fn pg_sleep_requires_the_builtin_name() {
    assert!(super::scan::is_pg_sleep(&["pg_sleep".into()]));
    assert!(super::scan::is_pg_sleep(&[
        "pg_catalog".into(),
        "pg_sleep".into(),
    ]));
    assert!(!super::scan::is_pg_sleep(&["PG_SLEEP".into()]));
    assert!(!super::scan::is_pg_sleep(&[
        "custom".into(),
        "pg_sleep".into(),
    ]));
}

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-test-state-mutation/fixture"),
    )
}

fn config(helper_roots: bool) -> NoMistakesConfig {
    let options = if helper_roots {
        "testGlobs: ['tests/**/*.test.ts']\nhelperRoots: ['helpers/**']\nimportSpecifier: '@example/db'"
    } else {
        "testGlobs: ['tests/**/*.test.ts']\nimportSpecifier: '@example/db'"
    };
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn files(root: &Path) -> Vec<PathBuf> {
    [
        "tests/mutations.test.ts",
        "helpers/schema-sql.ts",
        "helpers/reset.ts",
        "src/production.ts",
        "migrations/001.sql",
    ]
    .into_iter()
    .map(|rel| root.join(rel))
    .collect()
}

#[test]
fn test_globs_are_required_and_validated() {
    let root = fixture();
    let mut config = config(false);
    config.rules[0].options["testGlobs"] = serde_yaml::from_str("[]").unwrap();
    let error = check_with_files(&root, &config, &files(&root)).unwrap_err();
    assert!(error.to_string().contains("testGlobs must list"), "{error}");
    config.rules[0].options["testGlobs"] = serde_yaml::from_str("['[']").unwrap();
    let error = check_with_files(&root, &config, &files(&root)).unwrap_err();
    assert!(error.to_string().contains("invalid glob"), "{error}");
}

#[test]
fn line_suppression_uses_the_shared_rule_directive() {
    let root = fixture();
    let file = root.join("tests/suppressed.test.ts");
    let mut findings =
        check_with_files(&root, &config(false), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    let sources = super::super::source_store_for_files(&[file]);
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn advice_and_ignore_retain_definite_mutations_and_record_residuals() {
    let root = fixture();
    let files = vec![
        root.join("tests/variants.test.ts"),
        root.join("tests/dynamic.test.ts"),
        root.join("tests/malformed.test.ts"),
    ];
    let sources = super::super::source_store_for_files(&files);
    for mode in ["fail", "advise", "ignore"] {
        let mut config = config(false);
        config.rules[0].name = Some("test-database".into());
        config.rules[0].options["unanalyzableSql"] = mode.into();
        let facts = crate::codebase::postgres::prepare_rule_sql_facts(
            &root,
            &files,
            Arc::clone(&sources),
            &config,
            &[RULE_ID],
        )
        .unwrap()
        .with_fresh_sql_verdicts();
        let _findings =
            check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
        let verdicts = facts.sql_verdicts();
        assert!(!verdicts.is_empty());
        assert!(verdicts
            .iter()
            .any(|verdict| verdict.status == crate::codebase::postgres::SqlVerdictStatus::Abstain));
        assert!(verdicts.iter().any(|verdict| verdict.advisory) == (mode == "advise"));
    }
}

#[test]
fn invalid_policy_and_missing_prepared_facts_propagate_actionable_errors() {
    let root = fixture();
    let mut config = config(false);
    config.rules[0].options["unanalyzableSql"] = "maybe".into();
    let error = check_with_files(&root, &config, &[]).unwrap_err();
    assert!(error.to_string().contains("unanalyzableSql"), "{error}");
    config.rules[0].options["unanalyzableSql"] = "fail".into();
    let files = [root.join("tests/dynamic.test.ts")];
    let sources = super::super::source_store_for_files(&files);
    let error =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &Default::default())
            .unwrap_err();
    assert!(error.to_string().contains("dynamic.test.ts"), "{error}");
}
