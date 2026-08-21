use super::{nth_create_index_line, nth_drop_index_line, nth_drop_table_line};

#[test]
fn multiline_create_index_uses_the_create_keyword_line() {
    let sql = "SELECT 1;\nCREATE INDEX\n  idx_name\n  ON t (a);";
    assert_eq!(nth_create_index_line(sql, 1), 2);
}

#[test]
fn later_create_index_uses_occurrence_not_the_first_name() {
    let sql = "CREATE INDEX idx ON t (a);\nCREATE UNIQUE INDEX idx ON t (a, b);";
    assert_eq!(nth_create_index_line(sql, 1), 1);
    assert_eq!(nth_create_index_line(sql, 2), 2);
}

#[test]
fn comments_and_string_literals_are_not_create_index_occurrences() {
    let sql =
        "-- CREATE INDEX fake ON t (a);\nSELECT 'CREATE INDEX also';\nCREATE INDEX real ON t (a);";
    assert_eq!(nth_create_index_line(sql, 1), 3);
}

#[test]
fn block_comments_and_escaped_quotes_are_skipped() {
    let sql = "/* CREATE INDEX hidden ON t (a);\n*/\nCREATE INDEX real ON t (a, '''x');";
    assert_eq!(nth_create_index_line(sql, 1), 3);
}

#[test]
fn missing_occurrence_falls_back_to_line_one() {
    assert_eq!(nth_create_index_line("SELECT 1;", 1), 1);
    assert_eq!(nth_drop_index_line("SELECT 1;", 1), 1);
    assert_eq!(nth_drop_table_line("SELECT 1;", 1), 1);
}

#[test]
fn drop_keywords_can_span_lines() {
    let sql = "DROP INDEX\n  public.idx;\nDROP TABLE\n  public.t;";
    assert_eq!(nth_drop_index_line(sql, 1), 1);
    assert_eq!(nth_drop_table_line(sql, 1), 3);
}

#[test]
fn second_drop_uses_occurrence_not_the_first_match() {
    let sql = "DROP INDEX first;\nDROP INDEX second;\nDROP TABLE first;\nDROP TABLE second;";
    assert_eq!(nth_drop_index_line(sql, 2), 2);
    assert_eq!(nth_drop_table_line(sql, 2), 4);
}

#[test]
fn quoted_strings_may_span_lines() {
    let sql = "SELECT 'CREATE INDEX fake\nON t (a)';\nCREATE INDEX real ON t (a);";
    assert_eq!(nth_create_index_line(sql, 1), 3);
}
