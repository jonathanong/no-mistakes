use super::*;

#[test]
fn unrelated_switches_preserve_existing_versions_and_single_value_parity() {
    let calls = calls("variants-unaffected-switch.ts");
    assert_eq!(calls.len(), 14);
    assert_eq!(texts(&calls[0]), ["SELECT 1", "SELECT 2"]);
    for call in &calls[1..4] {
        assert_eq!(call.kind, EmbeddedSqlKind::ImmutableLocal);
        assert_eq!(call.sql_text.as_deref(), Some("SELECT 3"));
        assert!(call.variants.is_empty(), "{call:#?}");
        assert_eq!(call.sql_source_positions, calls[1].sql_source_positions);
        assert_eq!(
            call.recovered_placeholder_positions,
            calls[1].recovered_placeholder_positions
        );
    }
    assert_eq!(texts(&calls[4]), ["SELECT 4", "SELECT 5"]);
    assert_eq!(texts(&calls[5]), ["SELECT 4", "SELECT 5"]);
    for call in &calls[11..14] {
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic);
        assert_eq!(call.sql_text.as_deref(), Some("SELECT 45"));
        assert!(call.variants.is_empty(), "{call:#?}");
        assert_eq!(call.sql_source_positions, calls[11].sql_source_positions);
    }
    assert_eq!(calls, self::calls("variants-unaffected-switch.ts"));
}

#[test]
fn changed_switch_bindings_keep_correlations_and_the_real_version_cap() {
    let calls = calls("variants-unaffected-switch.ts");
    let mut correlated = texts(&calls[6]);
    correlated.sort();
    assert_eq!(correlated, ["SELECT 0", "SELECT 61", "SELECT 72"]);
    let mut fallthrough = texts(&calls[7]);
    fallthrough.sort();
    assert_eq!(fallthrough, ["SELECT 3", "SELECT 83", "SELECT 94"]);
    assert_eq!(calls[8].variants.len(), MAX_EMBEDDED_SQL_VARIANTS);
    assert!(calls[9].is_unanalyzable(), "{:#?}", calls[9]);
    assert!(calls[10].is_unanalyzable(), "{:#?}", calls[10]);
}
