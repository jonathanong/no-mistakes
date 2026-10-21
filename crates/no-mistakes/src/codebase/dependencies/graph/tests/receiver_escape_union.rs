use super::{assert_export, assert_possible_export, assert_unknown, graph};

#[test]
fn guarded_array_unions_slots_by_position() {
    let (built, root) = graph();
    // Slot 0 is equal on both paths; slot 1 exists only on the guarded path.
    // Both origins already reach the historical latest-array projection.
    assert_unknown(&built, &root, "array-reassign-union", "replica.get");
    assert_possible_export(&built, &root, "array-reassign-union", "untouched.get");
    for callee in ["primary.get", "unrelated.get"] {
        assert_export(&built, &root, "array-reassign-union", callee);
    }
}
