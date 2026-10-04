fn fixture() -> std::path::PathBuf {
    crate::test_support::rule_fixture_root("postgres-explicit-columns").join("mutation-table")
}

#[test]
fn mutation_table_arms_preserve_identity_locations_and_exists_star_policy() {
    let sql = std::fs::read_to_string(fixture().join("sql/mutations.sql")).unwrap();
    let facts = crate::codebase::postgres::extract_sql_statement_facts(&sql);
    assert!(!facts.parse_failed);
    let arms: Vec<_> = facts
        .selects
        .iter()
        .filter(|select| {
            select
                .tables
                .iter()
                .any(|table| table.starts_with("public."))
        })
        .collect();
    assert_eq!(
        arms.iter()
            .map(|fact| (
                fact.tables[0].as_str(),
                fact.line,
                fact.star_projections.len()
            ))
            .collect::<Vec<_>>(),
        [
            ("public.topics", 2, 0),
            ("public.\"Topics\"", 3, 0),
            ("public.topics", 4, 1),
            ("public.topics", 5, 1),
            ("public.topics", 6, 1),
            ("public.topics", 7, 1),
            ("public.topics", 11, 0)
        ]
    );
    assert!(facts
        .selects
        .iter()
        .all(|select| !select.tables.iter().any(|table| table == "topics")));
    assert_eq!(facts.updates.len(), 4);
    assert_eq!(facts.deletes.len(), 4);
    assert_eq!(facts.bounds.len(), 8);
    assert!(facts.limit_uses.is_empty());
}

#[test]
fn mutation_table_arms_feed_shared_physical_relation_rules() {
    use crate::codebase::rules::{postgres_explicit_columns, postgres_required_predicates};
    use crate::config::v2::{
        schema::{RuleDef, RuleScope},
        NoMistakesConfig,
    };
    let root = fixture();
    let files = [root.join("sql/mutations.sql"), root.join("schema.json")];
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let rule = |name: &str, options: &str| RuleDef {
        rule: name.into(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(options),
        ..Default::default()
    };
    let config=NoMistakesConfig {rules:vec![rule(postgres_explicit_columns::RULE_ID,"sqlInclude: ['sql/mutations.sql']\nschemaCatalogPath: schema.json\nrelations: [public.topics]"),rule(postgres_required_predicates::RULE_ID,"sqlInclude: ['sql/mutations.sql']\nrelations: [{table: public.topics, requireColumns: [id]}]")],..Default::default()};
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
    let explicit = postgres_explicit_columns::check_with_files_sources_and_facts(
        &root, &config, &files, &sources, &prepared,
    )
    .unwrap();
    assert_eq!(
        explicit
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [4, 5, 6, 7]
    );
    let required = postgres_required_predicates::check_with_files_sources_and_facts(
        &root, &config, &files, &sources, &prepared,
    )
    .unwrap();
    assert_eq!(
        required
            .iter()
            .map(|finding| finding.line)
            .collect::<Vec<_>>(),
        [2, 4, 5, 6, 7, 11]
    );
}
