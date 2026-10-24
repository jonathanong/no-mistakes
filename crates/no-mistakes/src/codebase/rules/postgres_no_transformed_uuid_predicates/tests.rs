use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
pub(super) fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-no-transformed-uuid-predicates/fixture"),
    )
}
pub(super) fn config(extra: &str) -> NoMistakesConfig {
    NoMistakesConfig { rules: vec![RuleDef {
        rule: RULE_ID.into(), scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(&format!(
            "schemaCatalogPath: schema.json\nsqlInclude: ['*.sql']\nimportSpecifier: db\nexecutorNames: [query]\n{extra}")),
        ..Default::default()
    }], ..Default::default() }
}
fn run(name: &str, extra: &str) -> Vec<RuleFinding> {
    let root = fixture();
    check_with_files(
        &root,
        &config(extra),
        &[root.join(name), root.join("schema.json")],
    )
    .unwrap()
}
#[test]
fn casts_functions_all_clauses_and_mutations_are_reported() {
    let findings = run("fail.sql", "");
    assert_eq!(findings.len(), 9, "{findings:#?}");
    for (line, clause) in [
        (1, "WHERE"),
        (4, "JOIN ON"),
        (5, "HAVING"),
        (6, "WHERE"),
        (7, "WHERE"),
    ] {
        assert!(
            findings
                .iter()
                .any(|finding| finding.line == line && finding.message.contains(clause)),
            "{findings:#?}"
        );
    }
    assert!(findings.iter().all(
        |finding| finding.message.contains("Cast the parameter to uuid")
            && finding.message.contains("precomputed bound")
    ));
}
#[test]
fn bare_columns_parameter_casts_projection_order_and_non_uuid_columns_are_clear() {
    assert!(run("pass.sql", "").is_empty());
}
#[test]
fn catalog_resolution_preserves_aliases_schema_and_ambiguity() {
    let findings = run("scope.sql", "unanalyzableSql: ignore\n");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 2, 11],
        "{findings:#?}"
    );
}
#[test]
fn unknown_owners_follow_the_unknown_sql_policy() {
    let findings = run("scope.sql", "");
    assert!(
        findings
            .iter()
            .any(|finding| finding.line == 4 && finding.target.as_deref() == Some("unanalyzable")),
        "{findings:#?}"
    );
}
#[test]
fn embedded_literals_share_the_prepared_statement_projection() {
    let findings = run("embedded.ts", "");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 3, 5],
        "{findings:#?}"
    );
}
#[test]
fn unrecoverable_embedded_sql_fails_or_is_ignored() {
    assert_eq!(run("unknown.ts", "").len(), 1);
    assert!(run("unknown.ts", "unanalyzableSql: ignore\n").is_empty());
}
#[test]
fn options_require_a_complete_schema_catalog() {
    assert!(compile_options(&Options::default()).is_err());
    let opts = Options {
        schema_catalog_path: "schema.json".into(),
        unanalyzable_sql: "invalid".into(),
        ..Default::default()
    };
    assert!(compile_options(&opts)
        .err()
        .unwrap()
        .to_string()
        .contains("unanalyzableSql"));
}

#[test]
fn low_effort_checks_the_same_direct_and_local_const_literals() {
    let root = fixture();
    let mut cfg = config("");
    cfg.effort = Some(crate::config::v2::Effort::Low);
    let files = [root.join("embedded.ts"), root.join("schema.json")];
    assert_eq!(
        check_with_files(&root, &cfg, &files).unwrap(),
        run("embedded.ts", "")
    );
}
#[test]
fn directives_suppress_the_configured_executor_call() {
    let root = fixture();
    let files = [root.join("suppressed.ts"), root.join("schema.json")];
    let sources = super::super::source_store_for_files(&files);
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        let mut cfg = config("");
        cfg.effort = Some(effort);
        let mut findings = check_with_files(&root, &cfg, &files).unwrap();
        super::super::suppress_rule_findings_with_sources(&root, &mut findings, &sources);
        assert!(findings.is_empty(), "{findings:#?}");
    }
}

#[test]
fn custom_messages_prefix_the_explanation_and_static_rewrite() {
    let root = fixture();
    let files = [root.join("embedded.ts"), root.join("schema.json")];
    for effort in [
        crate::config::v2::Effort::Low,
        crate::config::v2::Effort::Max,
    ] {
        let mut cfg = config("");
        cfg.effort = Some(effort);
        cfg.rules[0].message = Some("Keep identity lookups indexable".into());
        let findings = check_with_files(&root, &cfg, &files).unwrap();
        assert!(findings.iter().all(|finding| finding
            .message
            .starts_with("Keep identity lookups indexable:")
            && finding.message.contains("UUID column")
            && finding.message.contains("Cast the parameter")));
    }
}
