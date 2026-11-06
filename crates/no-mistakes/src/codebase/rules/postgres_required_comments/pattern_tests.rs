use crate::codebase::rules::{postgres_column_naming, postgres_finite_text_columns, RuleFinding};
use crate::config::v2::load_v2_config;
use std::path::PathBuf;

fn run(variant: &str) -> Vec<anyhow::Result<Vec<RuleFinding>>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/postgres-pattern-exceptions");
    let config = load_v2_config(&root, Some(&root.join(format!("{variant}.yml")))).unwrap();
    let files = [
        root.join("schema.json"),
        root.join("partial.json"),
        root.join("invalid.json"),
    ];
    vec![
        super::check_with_files(&root, &config, &files),
        postgres_column_naming::check_with_files(&root, &config, &files),
        postgres_finite_text_columns::check_with_files(&root, &config, &files),
    ]
}

#[test]
fn only_patterns_suppressing_findings_are_used_including_every_overlap() {
    for result in run("report") {
        let findings = result.unwrap();
        assert_eq!(findings.len(), 2, "{findings:?}");
        assert!(findings
            .iter()
            .all(|finding| finding.rule == "unused-allow-entry"));
        assert!(findings
            .iter()
            .any(|finding| finding.target.as_deref() == Some("^missing$")));
        assert!(findings
            .iter()
            .any(|finding| matches!(finding.target.as_deref(), Some("^good$" | "^clean$"))));
    }
}

#[test]
fn excluded_catalogs_do_not_establish_staleness() {
    for result in run("excluded") {
        assert!(result.unwrap().is_empty());
    }
}

#[test]
fn partial_catalogs_do_not_establish_staleness() {
    for findings in run("partial").into_iter().flatten() {
        assert!(findings
            .iter()
            .all(|finding| finding.rule != "unused-allow-entry"));
    }
}

#[test]
fn failed_catalogs_retain_the_catalog_error() {
    for result in run("failed") {
        assert!(result.is_err());
    }
}

#[test]
fn disabled_column_checks_do_not_establish_exemption_staleness() {
    for result in run("no-columns") {
        assert!(result.unwrap().is_empty());
    }
}

#[test]
fn positive_column_selectors_define_the_complete_exception_domain() {
    let results = run("column-selection");
    let findings = results.into_iter().next().unwrap().unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    assert!(findings
        .iter()
        .all(|finding| finding.rule == "unused-allow-entry"));
}
