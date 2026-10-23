use super::*;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use crate::diagnostics::{InvocationObserver, TimingKind};
use std::sync::Arc;

#[test]
fn trace_rule_records_only_verbose_observers() {
    let verbose = InvocationObserver::new(true);
    let sources = SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&[])),
        Some(verbose.clone()),
    );
    assert_eq!(trace_rule(&sources, "special", || 42), 42);
    let timings = verbose.snapshot().timings;
    assert_eq!(timings.len(), 1);
    assert_eq!(timings[0].label, "rules.special");
    assert_eq!(timings[0].kind, TimingKind::Parallel);

    for observer in [None, Some(InvocationObserver::new(false))] {
        let sources =
            SourceStore::new_observed(Arc::new(FileInventory::from_paths(&[])), observer.clone());
        assert_eq!(trace_rule(&sources, "untimed", || 7), 7);
        assert!(observer.is_none_or(|observer| observer.snapshot().timings.is_empty()));
    }
}

#[test]
fn optional_fact_adapter_prepares_postgres_diagnostics_once() {
    use crate::config::v2::{
        schema::{RuleDef, RuleScope},
        NoMistakesConfig,
    };
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("undeclared-executors");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let files = vec![root.join("query.mts")];
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::from_paths(root, &files);
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: "postgres-require-query-annotation".into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(
            "importSpecifier: '@example/db'\nexecutorTypeNames: [Executor]",
        )
        .unwrap(),
        ..Default::default()
    });
    let reference_sources = snapshot.source_store_for(root);
    let facts = super::super::entrypoints::prepare_call_site_facts(
        root,
        &config,
        &files,
        &reference_sources,
    )
    .unwrap()
    .unwrap();
    let prepared = |sources| PreparedFilesystemRuleInputs {
        snapshot: &snapshot,
        next_config_resolver: None,
        vitest_catalog: None,
        sources,
        workflow_documents: None,
        tsconfig_gate_project_inputs: None,
        config_path: None,
    };
    let expected = run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
        root,
        &config,
        &files,
        prepared(reference_sources),
        Some(&facts),
    )
    .unwrap();
    assert!(expected
        .iter()
        .any(|finding| finding.target.as_deref() == Some("undeclared-executor-type")));
    let sources = snapshot.source_store_for(root);
    crate::ast::begin_parse_count(root);
    let actual = run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
        root,
        &config,
        &files,
        prepared(sources.clone()),
        None,
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(root);
    assert_eq!(actual, expected);
    assert_eq!(counts.get(&files[0]), Some(&1), "{counts:?}");
    assert_eq!(sources.physical_read_count(), 1);
}
