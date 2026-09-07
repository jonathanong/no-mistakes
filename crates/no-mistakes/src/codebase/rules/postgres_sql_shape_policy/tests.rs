use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-sql-shape-policy/fixture")
            .join(name),
    )
}

fn config() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("sqlInclude: [\"sql/**/*.sql\"]").unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn run(root: &Path) -> Vec<RuleFinding> {
    check_with_files(root, &config(), &[root.join("sql/001.sql")]).unwrap()
}

fn config_yaml(yaml: &str) -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.to_string(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(yaml).unwrap(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn flags_correlated_exists_union() {
    let findings = run(&fixture("fail"));
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(
        findings[0].target.as_deref(),
        Some("correlated-exists-set-operation")
    );
}

#[test]
fn restricted_union_is_clean() {
    assert!(run(&fixture("pass")).is_empty());
}

#[test]
fn uncorrelated_and_derived_unions_are_clean() {
    let root = fixture("pass");
    for name in ["uncorrelated.sql", "probe.sql", "derived.sql"] {
        let file = root.join("sql").join(name);
        let findings = check_with_files(&root, &config(), std::slice::from_ref(&file)).unwrap();
        assert!(findings.is_empty(), "{name} {findings:?}");
    }
}

#[test]
fn restricted_correlated_union_is_still_flagged() {
    let root = fixture("fail");
    let file = root.join("sql/restricted-correlated.sql");
    let findings = check_with_files(&root, &config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
}

#[test]
fn honors_disable_comments() {
    let root = fixture("fail");
    let file = root.join("sql/disabled.sql");
    let mut findings = check_with_files(&root, &config(), std::slice::from_ref(&file)).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    let sources = super::super::source_store_for_files(std::slice::from_ref(&file));
    super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
    assert!(findings.is_empty(), "{findings:#?}");
}

#[test]
fn compile_options_default_to_exists_set_op() {
    let compiled = compile_options(&Options::default()).unwrap();
    assert!(compiled.ban_exists_set_op);
    assert!(compiled.fail_unanalyzable);
}

#[test]
fn missing_source_file_errors() {
    let root = fixture("fail");
    let missing = root.join("sql/does-not-exist.sql");
    let error = check_with_files(&root, &config(), &[missing]).expect_err("read");
    assert!(
        error.to_string().contains("failed to collect PostgreSQL"),
        "{error}"
    );
}

#[test]
fn rejects_unknown_options() {
    let error = compile_options(&Options {
        unanalyzable_sql: "fial".into(),
        ..Default::default()
    })
    .err()
    .expect("mode");
    assert!(error.to_string().contains("unanalyzableSql"), "{error}");
    let error = compile_options(&Options {
        banned_shapes: vec!["other-shape".into()],
        ..Default::default()
    })
    .err()
    .expect("shape");
    assert!(error.to_string().contains("bannedShapes"), "{error}");
    assert!(
        !compile_options(&Options {
            banned_shapes: vec!["correlated-exists-set-operation".into()],
            unanalyzable_sql: "ignore".into(),
            ..Default::default()
        })
        .unwrap()
        .fail_unanalyzable
    );
}

#[test]
fn include_exclude_and_option_overrides() {
    let root = fixture("fail");
    let sql = root.join("sql/001.sql");
    assert!(check_with_files(
        &root,
        &config_yaml("sqlInclude: [\"sql/**/*.sql\"]\nexclude: ['sql/001.sql']"),
        std::slice::from_ref(&sql),
    )
    .unwrap()
    .is_empty());
    assert_eq!(
        check_with_files(
            &root,
            &config_yaml("sqlInclude: [\"sql/**/*.sql\"]\ninclude: ['sql/001.sql']"),
            std::slice::from_ref(&sql),
        )
        .unwrap()
        .len(),
        1
    );
    let error = check_with_files(
        &root,
        &config_yaml("include: ['[']"),
        std::slice::from_ref(&sql),
    )
    .expect_err("glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
    let error = check_with_files(
        &root,
        &config_yaml("exclude: ['[']"),
        std::slice::from_ref(&sql),
    )
    .expect_err("glob");
    assert!(error.to_string().contains("invalid glob"), "{error}");
    let compiled = compile_options(&Options {
        sql_include: vec!["migrations/**/*.sql".into()],
        import_specifier: "@other/db".into(),
        executor_names: vec!["run".into()],
        unanalyzable_sql: "ignore".into(),
        ..Default::default()
    })
    .unwrap();
    assert!(!compiled.fail_unanalyzable);
    assert!(compiled.ban_exists_set_op);
    assert_eq!(compiled.schema.sql_include, ["migrations/**/*.sql"]);
    assert_eq!(compiled.embedded.import_specifier, "@other/db");
    assert_eq!(compiled.embedded.executor_names, ["run"]);
}

#[test]
fn dynamic_and_unparseable_sql_are_fail_closed() {
    let dynamic = fixture("fail-dynamic");
    let ts = dynamic.join("src/query.ts");
    let flagged =
        check_with_files(&dynamic, &config_yaml("{}"), std::slice::from_ref(&ts)).unwrap();
    assert!(
        flagged
            .iter()
            .any(|finding| finding.message.contains("not statically recoverable")),
        "{flagged:?}"
    );
    let ignored =
        check_with_files(&dynamic, &config_yaml("unanalyzableSql: ignore"), &[ts]).unwrap();
    assert!(ignored.is_empty(), "{ignored:?}");
    let root = fixture("fail");
    let unparseable =
        check_with_files(&root, &config(), &[root.join("sql/unparseable.sql")]).unwrap();
    assert!(
        unparseable
            .iter()
            .any(|finding| finding.message.contains("could not be analyzed")),
        "{unparseable:?}"
    );
}
