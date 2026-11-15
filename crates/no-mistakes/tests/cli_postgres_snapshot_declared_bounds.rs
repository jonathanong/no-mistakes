use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_bounded_declared_bounds_check_json_snapshot() {
    let root = rules_path("postgres-bounded-statements", "declared-bounds", "fixture");
    assert_saved_fixture(
        "postgres_bounded_declared_bounds",
        &[("declared-bounds", root.clone(), root.join("valid.yml"))],
    );
}
