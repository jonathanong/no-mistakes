use super::*;
#[test]
fn imported_helper_projection_reuses_prepared_call_export_and_source_facts() {
    let saved = crate::test_support::materialize_saved_fixture(&fixture("helper-imported-written"));
    let root = crate::codebase::ts_resolver::normalize_path(saved.path());
    assert_eq!(root, crate::codebase::ts_resolver::normalize_path(&root));
    let caller = root.join("src/query.ts");
    let helper = root.join("src/helper.ts");
    let files = [caller.clone(), helper.clone()];
    let options: Options = config().rules[0].try_rule_options().unwrap();
    let profile = compile(&options).unwrap().embedded;
    let source = std::fs::read_to_string(&helper).unwrap();
    let unprojected =
        crate::codebase::postgres::extract_embedded_sql_from_source(&helper, &source, &profile);
    assert!(unprojected
        .calls
        .iter()
        .any(|call| call.line == 3 && !call.helper_definition));
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::ast::begin_parse_count(&root);
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        std::sync::Arc::clone(&sources),
        &config(),
        &[RULE_ID],
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    for file in &files {
        assert_eq!(counts.get(file), Some(&1), "{counts:?}");
    }
    assert_eq!(sources.physical_read_count(), 2);
    let caller_facts = facts.ts.get(&caller).unwrap();
    assert!(caller_facts
        .ts
        .imported_bindings
        .iter()
        .any(|binding| binding.local == "acquireUser"
            && binding.imported == "lockUser"
            && binding.specifier == "./helper"));
    let helper_facts = facts.ts.get(&helper).unwrap();
    assert!(helper_facts
        .ts
        .exported_bindings
        .iter()
        .any(|binding| binding.local == "lockUser" && binding.exported == "lockUser"));
    let summary = facts.embedded_sql(&helper, &profile).unwrap();
    assert!(summary.lock_helper_summaries.contains_key("lockUser"));
    assert!(summary
        .calls
        .iter()
        .any(|call| call.line == 3 && call.helper_definition));
    let caller_sql = facts.embedded_sql(&caller, &profile).unwrap();
    assert!(caller_sql.imported_lock_helpers.is_empty());
    let call = caller_sql.calls.iter().find(|call| call.line == 5).unwrap();
    assert!(!call.is_unanalyzable(), "{call:?}");
    assert!(call.executor_transaction_known && call.transaction_id.is_some());
    let provenance = call.lock_helper.as_ref().unwrap();
    assert_eq!(provenance.0, "acquireUser");
    assert_eq!(provenance.1, 3);
}
