use super::{contains_ignore_ascii_case, line_for_write};

#[test]
fn write_line_prefers_column_then_table_then_one() {
    assert_eq!(
        line_for_write(
            "UPDATE items SET created_at = now();\n",
            "items",
            "created_at"
        ),
        1
    );
    assert_eq!(
        line_for_write("MERGE INTO Items t\n", "items", "missing"),
        1
    );
    assert_eq!(line_for_write("SELECT 1;\n", "items", "created_at"), 1);
    assert!(contains_ignore_ascii_case("Created_At", "created_at"));
}
