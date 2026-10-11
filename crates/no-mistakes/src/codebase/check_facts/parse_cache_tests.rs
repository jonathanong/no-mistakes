use super::{collect_check_facts, CheckFactPlan};
use crate::codebase::ts_resolver::normalize_path;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn check_fact_collection_parses_each_path_once() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/shared-facts/fixture"),
    );
    let helper = normalize_path(&root.join("src/helper.ts"));
    let widget = normalize_path(&root.join("src/widget.tsx"));
    crate::ast::begin_parse_count(&root);
    crate::ast::with_request_parse_cache(|| {
        let facts = collect_check_facts(
            &root,
            vec![helper.clone(), widget.clone()],
            CheckFactPlan {
                imports: true,
                ..CheckFactPlan::default()
            },
        );
        assert!(facts.ts.get(&helper).is_some());
        assert!(facts.ts.get(&widget).is_some());
        assert_eq!(
            crate::ast::request_parse_cache_len(),
            0,
            "parallel check fact collection evicts per-file parse cache entries"
        );
    });
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.get(&helper), Some(&1), "{counts:#?}");
    assert_eq!(counts.get(&widget), Some(&1), "{counts:#?}");
}

#[test]
fn embedded_and_helper_projection_share_the_single_ts_parse() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-shared-snapshots");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let paths = ["src/query.mts", "src/db.mts"].map(|name| normalize_path(&root.join(name)));
    crate::ast::begin_parse_count(&root);
    let facts = collect_check_facts(
        &root,
        paths.to_vec(),
        CheckFactPlan {
            imports: true,
            symbols: true,
            embedded_sql: true,
            query_annotation: true,
            embedded_sql_options: vec![crate::codebase::postgres::EmbeddedSqlOptions::configured(
                "@app/db",
                &[],
            )],
            ..Default::default()
        },
    );
    let counts = crate::ast::finish_parse_count(&root);
    for path in paths {
        assert_eq!(counts.get(&path), Some(&1), "{counts:#?}");
        let file = facts.ts.get(&path).unwrap();
        assert!(file.symbols.is_some());
        assert_eq!(file.embedded_sql.len(), 1);
        assert_eq!(file.query_annotation.len(), 1);
    }
}

#[test]
fn annotation_projects_only_its_profile_without_losing_imported_helper_calls() {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("helper-tracing-reachability");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = normalize_path(fixture.path());
    let files = crate::codebase::ts_source::discover_visible_paths(&root);
    let mut config = NoMistakesConfig::default();
    for (rule, options) in [
        (
            "postgres-require-query-annotation",
            "importSpecifier: '@app/db'\ninclude: ['src/helper.mts']\nunanalyzableSql: ignore",
        ),
        (
            "postgres-conflict-ordering",
            "importSpecifier: '@app/db'\nexecutorNames: [write]",
        ),
        (
            "postgres-require-query-annotation",
            "importSpecifier: '@app/db'\ninclude: ['src/idle.mts']\nunanalyzableSql: ignore",
        ),
    ] {
        config.rules.push(RuleDef {
            rule: rule.into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str(options).unwrap(),
            ..Default::default()
        });
    }
    let mut plan = CheckFactPlan {
        embedded_sql: true,
        embedded_sql_options:
            crate::codebase::postgres::configured_embedded_sql_options_for_checks(&config).unwrap(),
        ..Default::default()
    };
    crate::codebase::postgres::configure_prepared_postgres_plan(&config, &mut plan).unwrap();
    assert_eq!(plan.embedded_sql_options.len(), 2);
    assert_eq!(plan.query_annotation_options.as_ref().unwrap().len(), 1);
    let sources = Arc::new(crate::codebase::ts_source::SourceStore::new(Arc::new(
        crate::codebase::ts_source::FileInventory::from_paths(&files),
    )));
    let facts = super::collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        files.clone(),
        Vec::new(),
        plan,
        None,
        Arc::clone(&sources),
    );
    let helper = normalize_path(&root.join("src/helper.mts"));
    let helper_facts = facts.ts.get(&helper).unwrap();
    assert_eq!(helper_facts.embedded_sql.len(), 2);
    assert_eq!(helper_facts.query_annotation.len(), 1);
    assert_eq!(
        helper_facts.query_annotation[0].0.executor_names,
        ["query", "read", "write"]
    );
    let annotation_profile = &helper_facts.query_annotation[0].0;
    let call_span = helper_facts
        .embedded_sql
        .iter()
        .find(|(profile, _)| profile == annotation_profile)
        .unwrap()
        .1
        .call_spans[0];
    assert_eq!(
        facts.query_annotation_prefix(&helper, annotation_profile, call_span),
        Some(&Some("SELECT 1".to_string()))
    );
    let findings = crate::codebase::rules::postgres_require_query_annotation::
        check_with_files_sources_and_facts(&root, &config, &files, &sources, &facts).unwrap();
    assert_eq!(findings.len(), 1, "{findings:#?}");
    assert_eq!(findings[0].file, "src/helper.mts");
    assert_eq!(findings[0].line, 3);
}
