#[test]
fn b_uses_the_shared_library() {
    assert_eq!(app::value(), 42);
}
