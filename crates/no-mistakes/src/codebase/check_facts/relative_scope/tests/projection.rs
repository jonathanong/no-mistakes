use super::*;

#[test]
fn unseeded_projection_keeps_only_the_configured_package() {
    let root = fixture_root();
    let inside = normalize_path(&root.join("packages/db/src/orders/lock.ts"));
    let outside = normalize_path(&root.join("packages/other/src/orders/lock.ts"));
    let transaction = normalize_path(&root.join("packages/db/src/transaction.ts"));
    let map = collect_check_facts_with_graph_files_playwright_and_sources(
        &root,
        vec![inside.clone(), outside.clone(), transaction.clone()],
        Vec::new(),
        plan(),
        None,
        sources_for(&root),
    );
    let inside_sql = sql_of(&map, &inside);
    assert!(
        inside_sql.iter().any(|sql| sql.contains("FROM orders")),
        "{inside_sql:?}"
    );
    assert!(
        inside_sql.iter().any(|sql| sql.contains("FROM accounts")),
        "{inside_sql:?}"
    );
    let inside_facts = map.embedded_sql(&inside, &options()).expect("inside");
    assert_eq!(inside_facts.matched_factory_names, ["openTransaction"]);
    assert_eq!(inside_facts.matched_type_names, ["TxExecutor"]);
    assert!(inside_facts.pending_relative.candidates.is_empty());
    let outside_facts = map.embedded_sql(&outside, &options()).expect("outside");
    assert!(outside_facts.calls.is_empty(), "{:?}", outside_facts.calls);
    assert!(outside_facts.matched_factory_names.is_empty());
    assert!(outside_facts.matched_type_names.is_empty());
    assert!(map
        .embedded_sql(&transaction, &options())
        .expect("transaction")
        .calls
        .is_empty());
}

#[test]
fn seeded_session_projection_uses_the_request_store() {
    let root = fixture_root();
    let session = AnalysisSession::disabled();
    let sources = session.visible_paths(&root).source_store_for(&root);
    let inside = normalize_path(&root.join("packages/db/src/orders/lock.ts"));
    let outside = normalize_path(&root.join("packages/other/src/orders/lock.ts"));
    let map = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (vec![inside.clone(), outside.clone()], Vec::new()),
        plan(),
        None,
        sources,
    );
    assert_eq!(sql_of(&map, &inside).len(), 2);
    assert!(sql_of(&map, &outside).is_empty());
}
