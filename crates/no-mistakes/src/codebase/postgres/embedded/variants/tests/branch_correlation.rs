use super::*;

#[test]
fn branch_assigned_guards_reuse_the_sql_paths_for_each_control_form() {
    let calls = calls("variants-branch-correlation.ts");
    assert_eq!(calls.len(), 12);
    for (call, expected) in calls.iter().take(11).zip([
        "SELECT 1",
        "SELECT 4",
        "SELECT 6",
        "SELECT 7",
        "SELECT 9",
        "SELECT 12",
        "SELECT 14",
        "SELECT 15",
        "SELECT 17 OFFSET 1",
        "SELECT 18",
        "SELECT 18",
    ]) {
        assert_eq!(texts(call), [expected], "{call:#?}");
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic);
    }
    assert_eq!(texts(&calls[11]), ["SELECT 19", "SELECT 20 OFFSET 1"]);
    assert_eq!(calls, self::calls("variants-branch-correlation.ts"));
}

#[test]
fn unreachable_guards_preserve_unrelated_legacy_bindings_and_skipped_mutations() {
    let calls = calls("variants-unchanged-guards.ts");
    assert_eq!(calls.len(), 9);
    assert_eq!(texts(&calls[0]), ["/* inner */ SELECT 2"]);
    for (call, sql) in calls[1..7].iter().zip([
        "SELECT 1", "SELECT 3", "SELECT 4", "SELECT 5", "SELECT 6", "SELECT 7",
    ]) {
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic);
        assert_eq!(call.sql_text.as_deref(), Some(sql));
        assert!(call.variants.is_empty(), "{call:#?}");
    }
    assert_eq!(texts(&calls[7]), ["SELECT 8"]);
    assert_eq!(texts(&calls[8]), ["SELECT 9"]);
}
