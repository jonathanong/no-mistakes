use super::*;

#[test]
fn a_direct_throw_excludes_only_its_unreachable_assignment_path() {
    let calls = calls("variants-branch-throw.ts");
    assert_eq!(calls.len(), 5);
    assert_eq!(
        texts(&calls[0]),
        ["CREATE TABLE finite_branch (id int)", "SELECT 1"]
    );
    for call in &calls[1..] {
        assert!(call.is_unanalyzable(), "{call:#?}");
    }
}
