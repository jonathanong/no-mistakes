use super::*;

#[test]
fn mutable_builder_loops_keep_zero_and_one_with_an_unknown_residual() {
    let calls = calls("variants-mutable-append-loops.ts");
    assert_eq!(calls.len(), 4);
    for (index, call) in calls.iter().take(3).enumerate() {
        assert_eq!(call.variants.len(), 2, "{call:#?}");
        assert_eq!(call.variants[0].sql_text, format!("SELECT {}", index + 1));
        assert!(call.variants_truncated());
        assert!(call.is_unanalyzable());
    }
    assert_eq!(texts(&calls[0]), ["SELECT 1", "SELECT 1 LIMIT 1"]);
    assert_eq!(
        texts(&calls[1]),
        ["SELECT 2", "SELECT 2 WHERE id = sql_placeholder_1"]
    );
    assert_eq!(
        calls[1].variants[1].recovered_placeholder_positions.len(),
        1
    );
    assert_eq!(texts(&calls[2]), ["SELECT 3", "SELECT 3 LIMIT 3"]);
    assert_eq!(calls[3].sql_text.as_deref(), Some("SELECT 4"));
    assert!(!calls[3].variants_truncated());
    assert!(!calls[3].is_unanalyzable());
}
