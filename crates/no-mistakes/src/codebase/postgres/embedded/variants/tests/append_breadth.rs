use super::*;

#[test]
fn append_budget_keeps_all_singletons_then_pairs_and_an_unknown_residual() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let file = crate::diagnostics::with_observer(Some(observer.clone()), || {
        file_facts("variants-append-breadth.ts")
    });
    assert_eq!(
        observer.snapshot().work["postgres.sql.variants_composed"],
        39
    );
    assert_eq!(observer.snapshot().work["postgres.sql.sites_truncated"], 2);
    let wide = &file.calls[0];
    assert_eq!(wide.variants.len(), 21);
    assert_eq!(wide.variants[0].sql_text, "/* bounded */ SELECT 1");
    for index in 0..20 {
        assert_eq!(
            wide.variants[index + 1].sql_text,
            format!("/* bounded */ SELECT 1 /* c{index:02} */")
        );
    }
    assert!(wide.variants_truncated());
    assert!(wide.is_unanalyzable());
    let projected: Vec<_> = wide.statement_calls().collect();
    assert_eq!(projected.len(), 21);
    assert!(projected.iter().all(|call| call.sql_text.is_some()
        && call.is_unanalyzable()
        && call.kind == EmbeddedSqlKind::Dynamic));

    let pairs = &file.calls[1];
    assert_eq!(pairs.variants.len(), 16);
    assert_eq!(pairs.variants[0].sql_text, "SELECT 2");
    assert_eq!(pairs.variants[5].sql_text, "SELECT 2 /* p4 */");
    assert_eq!(pairs.variants[6].sql_text, "SELECT 2 /* p0 */ /* p1 */");
    assert_eq!(pairs.variants[15].sql_text, "SELECT 2 /* p3 */ /* p4 */");
    assert!(pairs.variants_truncated());
    let prepared =
        crate::codebase::postgres::collect::dml::embedded_call_facts(&file, false, false);
    assert_eq!(prepared.len(), 39);
    assert!(prepared
        .iter()
        .all(|facts| facts.variant_locations.is_some()));
    assert_eq!(
        prepared.iter().filter(|facts| facts.parse_failed).count(),
        1
    );
}

#[test]
fn append_loops_sample_zero_and_one_and_mark_repeated_iterations_unknown() {
    let calls = calls("variants-append-loops.ts");
    assert_eq!(calls.len(), 6);
    for (index, call) in calls.iter().take(5).enumerate() {
        assert_eq!(call.variants.len(), 2, "{call:#?}");
        assert_eq!(call.variants[0].sql_text, format!("SELECT {}", index + 1));
        assert!(call.variants_truncated());
        assert!(call.is_unanalyzable());
    }
    assert_eq!(
        calls[2].variants[1].recovered_placeholder_positions.len(),
        1
    );
    assert_eq!(calls[5].sql_text.as_deref(), Some("SELECT 6"));
    assert!(!calls[5].variants_truncated());
    assert!(!calls[5].is_unanalyzable());
}

#[test]
fn template_strings_parameterizes_all_interpolations_and_composes_deep_chains() {
    let calls = calls("variants-append-interpolation.ts");
    assert_eq!(calls.len(), 7);
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("SELECT sql_placeholder_1 WHERE id = sql_placeholder_2 AND n = sql_placeholder_3")
    );
    assert_eq!(calls[0].recovered_placeholder_positions.len(), 3);
    assert_eq!(
        texts(&calls[1]),
        [
            "SELECT sql_placeholder_1",
            "SELECT sql_placeholder_1 WHERE id = sql_placeholder_2"
        ]
    );
    assert_eq!(texts(&calls[2]), ["SELECT 1", "SELECT 1 /* suffix */"]);
    assert_eq!(
        calls[3].sql_text.as_deref(),
        Some(format!("SELECT 7{}", " /* link */".repeat(20)).as_str())
    );
    assert!(!calls[3].is_unanalyzable());
    assert_eq!(
        calls[4].sql_text.as_deref(),
        Some("SELECT sql_placeholder_1")
    );
    assert_eq!(calls[5].sql_text.as_deref(), Some("SELECT 8 WHERE active"));
    assert_eq!(
        calls[6].sql_text.as_deref(),
        Some("SELECT sql_placeholder_1")
    );
}
