use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn value_const_root() -> PathBuf {
    let mut path = rules_path("postgres-bounded-statements", "value-const-holes", "src");
    path.pop();
    path
}

#[test]
fn postgres_value_const_holes_check_json_snapshot() {
    assert_saved_fixture(
        "postgres_value_const_holes",
        &[(
            "value_const_holes",
            value_const_root(),
            rules_path(
                "postgres-bounded-statements",
                "configs",
                "value-const-holes.yml",
            ),
        )],
    );
}
