use super::{plan, CheckFactPlan};
use crate::codebase::check_facts::collect_check_facts_with_graph_files_playwright_and_sources;
use crate::codebase::postgres::query_annotation::heads::HeadProof;
use crate::codebase::ts_source::{discover_visible_paths, FileInventory, SourceStore};
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use crate::diagnostics::{with_observer, InvocationObserver};
use std::sync::Arc;

fn collect(mixed: bool) -> (Vec<HeadProof>, Vec<u32>, u64) {
    let saved = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("relative-helper-heads");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path();
    let source_store = Arc::new(SourceStore::new(Arc::new(FileInventory::from_paths(
        &discover_visible_paths(root),
    ))));
    let config_path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .parent()
        .unwrap()
        .join("configs/relative-helper-heads-report.yml");
    let mut config: NoMistakesConfig =
        serde_yaml::from_str(&std::fs::read_to_string(config_path).unwrap()).unwrap();
    if mixed {
        config.rules.push(RuleDef {
            rule: "postgres-explicit-columns".into(),
            scope: Some(RuleScope::Repository),
            options: serde_yaml::from_str("importSpecifier: '@other/db'\nexecutorNames: [read]")
                .unwrap(),
            ..Default::default()
        });
    }
    crate::codebase::rules::analysis_policy::prepare_rule_analysis(
        root,
        &mut config,
        &[],
        &source_store,
    )
    .unwrap();
    let plan: CheckFactPlan = plan(root, &config);
    let query = root.join("packages/db/src/query.mts");
    let observer = InvocationObserver::new(true);
    let facts = with_observer(Some(observer.clone()), || {
        collect_check_facts_with_graph_files_playwright_and_sources(
            root,
            vec![
                query.clone(),
                root.join("packages/db/src/transaction.mts"),
                root.join("packages/other/src/transaction.mts"),
            ],
            Vec::new(),
            plan,
            None,
            source_store,
        )
    });
    let file = facts.ts.get(&query).expect("query prepared");
    let embedded = &file
        .embedded_sql
        .iter()
        .find(|(options, _)| options.import_specifier == "@example/db")
        .unwrap()
        .1;
    let heads = file.query_annotation_heads.as_ref().expect("low heads");
    let proofs = embedded
        .call_spans
        .iter()
        .map(|span| *heads.calls.get(span).unwrap())
        .collect();
    let lines = embedded.calls.iter().map(|call| call.line).collect();
    let composed = observer
        .snapshot()
        .work
        .get("postgres.sql.variants_composed")
        .copied()
        .unwrap_or(0);
    (proofs, lines, composed)
}

#[test]
fn relative_executor_projection_retains_only_confirmed_helper_heads() {
    let low = collect(false);
    let mixed = collect(true);
    assert_eq!(
        low.0,
        [
            HeadProof::Literal,
            HeadProof::Missing,
            HeadProof::Literal,
            HeadProof::Unproven
        ]
    );
    assert_eq!(low.1, [11, 12, 14, 20]);
    assert_eq!(low.2, 0, "low must not compose SQL variants");
    assert_eq!(mixed.0, low.0, "full demand cannot change low proofs");
    assert_eq!(mixed.1, low.1, "projection keeps call order");
}
