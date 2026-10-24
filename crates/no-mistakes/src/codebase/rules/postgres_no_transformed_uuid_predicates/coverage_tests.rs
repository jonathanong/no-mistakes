use super::tests::{config, fixture};
use super::*;
fn findings(name: &str, extra: &str) -> Vec<RuleFinding> {
    let root = fixture();
    check_with_files(
        &root,
        &config(extra),
        &[root.join(name), root.join("schema.json")],
    )
    .unwrap()
}
#[test]
fn quoted_names_keep_case_schema_dots_and_alias_identity() {
    let findings = findings("quoted.sql", "unanalyzableSql: ignore\n");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [1, 3, 4, 5, 6],
        "{findings:#?}"
    );
}
#[test]
fn nonrecursive_recursive_qualified_and_derived_cte_scopes_stay_distinct() {
    let findings = findings("ctes.sql", "unanalyzableSql: ignore\n");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 5, 6, 7, 8],
        "{findings:#?}"
    );
}
#[test]
fn empty_scope_missing_columns_and_derived_columns_are_unknown() {
    let findings = findings("unknown-scope.sql", "");
    assert_eq!(findings.len(), 7, "{findings:#?}");
    assert!(
        findings
            .iter()
            .all(|finding| finding.target.as_deref() == Some("unanalyzable")),
        "{findings:#?}"
    );
    assert!(self::findings("unknown-scope.sql", "unanalyzableSql: ignore\n").is_empty());
}
#[test]
fn nested_wrappers_mutation_sources_and_predicate_forms_are_checked() {
    let findings = findings("predicate-forms.sql", "");
    assert_eq!(findings.len(), 7, "{findings:#?}");
    assert!(findings
        .iter()
        .all(|finding| finding.target.as_deref() != Some("unanalyzable")));
}
#[test]
fn excludes_apply_to_sql_and_embedded_inputs() {
    assert!(findings("fail.sql", "exclude: ['*.sql']\n").is_empty());
    assert!(findings("embedded.ts", "exclude: ['*.ts']\n").is_empty());
}
#[test]
fn invalid_globs_fail_before_scanning() {
    for field in ["include", "exclude", "sqlInclude"] {
        let root = fixture();
        let mut cfg = config("");
        cfg.rules[0]
            .options
            .as_mapping_mut()
            .unwrap()
            .insert(field.into(), serde_yaml::from_str("['[']").unwrap());
        let result = check_with_files(&root, &cfg, &[root.join("schema.json")]);
        assert!(result.is_err(), "{field}");
    }
}
