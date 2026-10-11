use super::*;

#[test]
fn opaque_and_nullishness_stays_dynamic_without_losing_ordinary_fragment_versions() {
    let calls = calls("variants-opaque-and-nullish.ts");
    assert_eq!(calls.len(), 9);
    for index in [0, 1, 8] {
        assert!(
            calls[index].is_unanalyzable(),
            "{index}: {:#?}",
            calls[index]
        );
    }
    let mut optional = texts(&calls[2]);
    optional.sort();
    assert_eq!(optional, ["SELECT 3 ", "SELECT 3 LIMIT 1"]);
    let mut or_fallback = texts(&calls[3]);
    or_fallback.sort();
    assert_eq!(or_fallback, ["SELECT 4 LIMIT 1", "SELECT 4 LIMIT 2"]);
    for (index, expected) in [(4, "SELECT 5 "), (5, "SELECT 6 "), (6, "SELECT 7 ")] {
        assert_eq!(texts(&calls[index]), [expected]);
    }
    let mut mutated = texts(&calls[7]);
    mutated.sort();
    assert_eq!(mutated, ["SELECT 8", "SELECT 8 OFFSET 8"]);
    assert_eq!(calls, self::calls("variants-opaque-and-nullish.ts"));
}
