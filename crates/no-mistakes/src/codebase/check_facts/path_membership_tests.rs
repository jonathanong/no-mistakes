use super::{ordered_path_exclusion, ordered_path_intersection, PathMembership};
use std::path::PathBuf;

#[test]
fn indexed_membership_preserves_order_and_duplicates() {
    let scope = vec![PathBuf::from("a.ts"), PathBuf::from("b.ts")];
    let candidates = vec![
        PathBuf::from("b.ts"),
        PathBuf::from("missing.ts"),
        PathBuf::from("b.ts"),
        PathBuf::from("a.ts"),
    ];
    let membership = PathMembership::new(&scope);
    assert_eq!(membership.index.len(), 2);
    assert_eq!(
        ordered_path_intersection(&candidates, &scope),
        vec![
            PathBuf::from("b.ts"),
            PathBuf::from("b.ts"),
            PathBuf::from("a.ts")
        ]
    );
    assert_eq!(
        ordered_path_exclusion(
            &candidates,
            &[PathBuf::from("b.ts")],
            &[PathBuf::from("a.ts")]
        ),
        vec![PathBuf::from("missing.ts")]
    );
}
