use super::*;

#[test]
fn head_only_demand_preserves_executor_ownership_without_composing_sql() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/append-head-demand.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_scoped_executors(&["createPool".into()], &["DbClient".into()]);
    let full = extract_embedded_sql_from_source(&path, &source, &options);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let heads = crate::diagnostics::with_observer(Some(observer.clone()), || {
        crate::ast::with_program(&path, &source, |program, source| {
            extract_embedded_sql_from_program_with_demand(&path, program, source, &options, true)
        })
        .unwrap()
    });
    assert_eq!(heads.calls.len(), 4);
    assert_eq!(heads.call_spans, full.call_spans);
    assert_eq!(
        heads.unproven_member_query_spans,
        full.unproven_member_query_spans
    );
    assert_eq!(heads.unproven_member_query_spans.len(), 1);
    assert!(heads.calls.iter().all(|call| call.variants.is_empty()));
    assert!(heads
        .calls
        .iter()
        .all(|call| call.sql_source_positions.is_empty()));
    assert!(heads.fragments.is_empty());
    assert_eq!(
        heads.calls[0].sql_text.as_deref(),
        Some("/* named */ SELECT ")
    );
    assert_eq!(
        observer.snapshot().work["postgres.sql.variants_composed"],
        0
    );
    assert_eq!(observer.snapshot().work["postgres.sql.sites_truncated"], 0);
}

#[test]
fn head_only_diagnostic_text_respects_literal_forms_and_unknown_call_heads() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/append-head-edge-demand.ts");
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let heads = crate::ast::with_program(&path, &source, |program, source| {
        extract_embedded_sql_from_program_with_demand(&path, program, source, &options, true)
    })
    .unwrap();
    assert_eq!(heads.calls.len(), 6);
    for (call, expected) in heads.calls[..3].iter().zip([
        "/* template */ SELECT ",
        "/* raw */ SELECT \\n ",
        "/* tag */ SELECT ",
    ]) {
        assert_eq!(call.sql_text.as_deref(), Some(expected));
        assert!(call.variants.is_empty());
    }
    assert!(heads.calls[3].sql_text.is_none());
    assert!(heads.calls[4].sql_text.is_none());
    assert!(heads.calls.iter().all(|call| call.variants.is_empty()));
}
