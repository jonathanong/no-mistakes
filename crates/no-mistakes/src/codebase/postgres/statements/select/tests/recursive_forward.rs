fn fixture() -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("postgres-explicit-columns").join("recursive-forward")
}

#[test]
fn recursive_forward_aliases_are_local_in_select_and_dependency_facts() {
    let sql = std::fs::read_to_string(fixture().join("sql/forward.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    assert_eq!(facts.bounds.len(), 7);
    assert_eq!(
        facts
            .selects
            .iter()
            .flat_map(|select| select.tables.iter())
            .map(String::as_str)
            .collect::<Vec<_>>(),
        [
            "accounts",
            "accounts",
            "accounts",
            "accounts",
            "later_cte",
            "accounts",
            "public.later_cte",
            "accounts",
            "accounts",
            "accounts",
            "accounts",
            "accounts"
        ]
    );
    assert_eq!(
        facts
            .selects
            .iter()
            .flat_map(|select| &select.star_projections)
            .map(|star| (star.relation.as_str(), star.line))
            .collect::<Vec<_>>(),
        [("later_cte", 4), ("public.later_cte", 5)]
    );
}

#[test]
fn recursive_forward_aliases_do_not_trigger_physical_relation_rules() {
    use crate::codebase::rules::{postgres_explicit_columns, postgres_required_predicates};
    use crate::config::v2::{
        schema::{RuleDef, RuleScope},
        NoMistakesConfig,
    };
    let root = fixture();
    let files = [root.join("sql/forward.sql"), root.join("schema.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let rule = |name: &str, options: &str| RuleDef {
        rule: name.into(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(options),
        ..Default::default()
    };
    let config = NoMistakesConfig { rules: vec![rule(postgres_explicit_columns::RULE_ID, "sqlInclude: ['sql/forward.sql']\nschemaCatalogPath: schema.json\nrelations: [later_cte, public.later_cte]"), rule(postgres_required_predicates::RULE_ID, "sqlInclude: ['sql/forward.sql']\nrelations: [{table: later_cte, requireColumns: [id]}]")], ..Default::default() };
    let prepared = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        std::sync::Arc::clone(&sources),
        &config,
        &[
            postgres_explicit_columns::RULE_ID,
            postgres_required_predicates::RULE_ID,
        ],
    )
    .unwrap();
    for findings in [
        postgres_explicit_columns::check_with_files_sources_and_facts(
            &root, &config, &files, &sources, &prepared,
        )
        .unwrap(),
        postgres_required_predicates::check_with_files_sources_and_facts(
            &root, &config, &files, &sources, &prepared,
        )
        .unwrap(),
    ] {
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.line)
                .collect::<Vec<_>>(),
            [4, 5],
            "{findings:?}"
        );
    }
}
