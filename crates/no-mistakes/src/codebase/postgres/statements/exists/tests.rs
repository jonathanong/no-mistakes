#[test]
fn missing_exists_keyword_stays_at_the_first_column() {
    assert_eq!(super::exists_position("SELECT 1"), (1, 1));
    assert_eq!(super::exists_position(""), (1, 1));
}
