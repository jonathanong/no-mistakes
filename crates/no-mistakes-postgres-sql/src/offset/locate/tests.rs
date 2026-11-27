use super::Positions;

#[test]
fn indexed_positions_reject_invalid_locations_and_preserve_eof() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres/source-positions/fixture/offsets.sql"
    ));
    let positions = Positions::new(sql);
    assert_eq!(positions.index_at(0, 1), None);
    assert_eq!(positions.index_at(1, 0), None);
    assert_eq!(positions.index_at(99, 1), None);
    assert_eq!(positions.index_at(1, usize::MAX), None);
    assert_eq!(positions.index_at(2, usize::MAX), None);
    assert_eq!(positions.index_at(6, 1), Some(sql.len()));
    assert_eq!(positions.index_at(6, 2), None);
    assert_eq!(positions.index_at(6, usize::MAX), None);
    assert_eq!(
        positions.index_at(2, usize::MAX - positions.lines[1].1 + 1),
        None
    );
    assert_eq!(positions.line_col_at(0), Some((1, 1)));
    assert_eq!(positions.unicode.len(), 2);
    assert_eq!(positions.line_col_at(sql.len()), Some((6, 1)));
    assert_eq!(positions.line_col_at(sql.len() + 1), None);
    let unicode = sql.find('é').unwrap();
    assert_eq!(positions.line_col_at(unicode + 1), None);
    assert_eq!(positions.index_at(2, 9), Some(unicode));
}

#[test]
fn sparse_positions_round_trip_every_fixture_character() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/postgres/source-positions/fixture/offsets.sql"
    ));
    let positions = Positions::new(sql);
    let (mut line, mut column) = (1, 1);
    for (byte, character) in sql.char_indices() {
        assert_eq!(positions.index_at(line, column), Some(byte));
        assert_eq!(positions.line_col_at(byte), Some((line, column)));
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
}
