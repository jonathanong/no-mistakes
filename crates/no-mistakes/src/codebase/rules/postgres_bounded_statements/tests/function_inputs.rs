use super::{fixture_root, names};

#[test]
fn unknown_function_inputs_do_not_bound_table_functions() {
    let sql = std::fs::read_to_string(fixture_root().join("sql/function-inputs.sql")).unwrap();
    assert_eq!(names(&sql), ["accounts", "accounts", "accounts"]);
}
