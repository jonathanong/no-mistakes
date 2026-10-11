use super::*;

#[test]
fn equivalent_physical_branch_values_do_not_consume_the_concrete_version_cap() {
    let calls = calls("variants-cap-equivalent.ts");
    assert_eq!(calls.len(), 7);
    assert_eq!(
        texts(&calls[0]),
        ["SELECT 1 /* same */ /* same */ /* same */ /* same */ /* same */"]
    );
    let mut correlated = texts(&calls[1]);
    correlated.sort();
    assert_eq!(
        correlated,
        [
            "SELECT 2 /* same */ /* same */ /* same */ /* same */ /* same */ /* no */ /* false */",
            "SELECT 2 /* same */ /* same */ /* same */ /* same */ /* same */ /* yes */ /* true */",
        ]
    );
    let mut distinct = texts(&calls[2]);
    distinct.sort();
    assert_eq!(
        distinct,
        [
            "SELECT 3 /* left */ /* same */",
            "SELECT 3 /* same */ /* right */"
        ]
    );
    assert_eq!(
        texts(&calls[3]),
        ["SELECT 4 /* same */ /* same */ /* same */ /* same */ /* same */"]
    );
    let mut recovered_guard = texts(&calls[4]);
    recovered_guard.sort();
    assert_eq!(
        recovered_guard,
        [
            "SELECT 5 /* left */ /* left */ /* left */ /* left */ /* left */ /* true */",
            "SELECT 5 /* right */ /* right */ /* right */ /* right */ /* right */ /* false */",
        ]
    );
    assert_eq!(texts(&calls[5]), ["SELECT 6 /* same */"]);
    assert!(calls[6].is_unanalyzable());
    assert_eq!(calls, self::calls("variants-cap-equivalent.ts"));
}
