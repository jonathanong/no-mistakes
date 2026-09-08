use super::*;

#[test]
fn explicitly_empty_graph_file_universe_is_complete() {
    let root = fixture_path("");
    let scoped = fixture_path("src/everything.tsx");
    let facts = collect_check_facts_with_graph_files_and_playwright(
        &root,
        vec![scoped],
        Vec::new(),
        CheckFactPlan::default(),
        None,
    );

    assert_eq!(
        crate::codebase::dependencies::graph::TsFactLookup::graph_files(&facts),
        Some([].as_slice())
    );
    assert!(facts.graph_file_universe_is_complete());
    assert!(facts.graph_file_universe().is_empty());
}

#[test]
fn collect_check_facts_only_parses_playwright_test_files_for_playwright_facts() {
    let root = fixture_path("");
    let test_file = fixture_path("src/everything.tsx");
    let invalid_file = fixture_path("src/invalid.ts");
    let facts = collect_check_facts_with_playwright(
        &root,
        vec![test_file.clone(), invalid_file.clone()],
        CheckFactPlan::default(),
        Some(playwright_plan(test_file.clone())),
    );

    assert_eq!(facts.stats.files_discovered, 2);
    assert_eq!(facts.stats.files_parsed, 1);
    assert_eq!(facts.stats.parse_errors, 0);
    assert!(facts
        .ts
        .get(&test_file)
        .expect("test file facts")
        .playwright
        .is_some());
    assert!(!facts.ts.contains_key(&invalid_file));
}

#[test]
fn playwright_fact_plan_union_preserves_staged_variants_and_source_metadata() {
    let first = fixture_path("src/everything.tsx");
    let second = fixture_path("src/widget.tsx");
    let mut plan = playwright_plan(first.clone());
    plan.set_source_files(vec![first.clone()]);
    let mut other = playwright_plan(second.clone());
    other.set_source_files(vec![second.clone()]);

    plan.include(other);

    assert!(plan.file(&first).is_some());
    assert!(plan.file(&second).is_some());
    assert_eq!(
        plan.source_files().as_ref(),
        &[
            crate::codebase::ts_resolver::normalize_path(&first),
            crate::codebase::ts_resolver::normalize_path(&second),
        ]
    );
}
