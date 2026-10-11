use super::*;

#[test]
fn and_retains_nullishness_for_outer_fallbacks_without_treating_false_as_null() {
    let calls = calls("variants-logical-nullish.ts");
    assert_eq!(calls.len(), 10);
    for (index, mut expected) in [
        (0, vec!["SELECT 2", "SELECT 3"]),
        (1, vec!["SELECT 5", "SELECT 6"]),
        (2, vec!["SELECT 7 LIMIT 1", "SELECT 7 LIMIT 2"]),
        (4, vec!["SELECT 12", "SELECT 13"]),
        (5, vec!["SELECT 15"]),
        (7, vec!["SELECT 19", "SELECT 20"]),
    ] {
        let mut actual = texts(&calls[index]);
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "{index}: {:#?}", calls[index]);
    }
    for index in [3, 6, 8, 9] {
        assert!(
            calls[index].is_unanalyzable(),
            "{index}: {:#?}",
            calls[index]
        );
    }
    assert_eq!(calls, self::calls("variants-logical-nullish.ts"));
}
