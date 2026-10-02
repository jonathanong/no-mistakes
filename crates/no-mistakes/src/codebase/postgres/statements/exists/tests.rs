use sqlparser::tokenizer::Location;

#[test]
fn missing_exists_keyword_stays_at_the_first_column() {
    let none = Location { line: 0, column: 0 };
    assert_eq!(super::exists_position("SELECT 1", none), (1, 1));
    assert_eq!(super::exists_position("", none), (1, 1));
    let start = Location { line: 1, column: 5 };
    assert_eq!(super::exists_position("SELECT 1", start), (1, 1));
}

#[test]
fn each_exists_uses_its_own_keyword_position() {
    // The first EXISTS is in a string; the second is the one being located.
    let sql = "SELECT 'exists'\nWHERE EXISTS (SELECT 1)";
    let start = Location {
        line: 2,
        column: 15,
    };
    assert_eq!(super::exists_position(sql, start), (2, 7));
}
