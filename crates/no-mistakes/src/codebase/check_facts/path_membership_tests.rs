use super::{
    into_ordered_path_exclusion, ordered_path_exclusion, ordered_path_intersection, PathMembership,
};
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
    assert!(ordered_path_exclusion(&[], &scope, &candidates).is_empty());

    let retained = PathBuf::from("retained-path-with-an-owned-allocation.ts");
    let retained_allocation = retained.as_os_str().as_encoded_bytes().as_ptr();
    let moved = into_ordered_path_exclusion(vec![retained, PathBuf::from("b.ts")], &scope, &[]);
    assert_eq!(
        moved,
        vec![PathBuf::from("retained-path-with-an-owned-allocation.ts")]
    );
    assert_eq!(
        moved[0].as_os_str().as_encoded_bytes().as_ptr(),
        retained_allocation,
        "the consuming helper must retain the existing PathBuf allocation"
    );
    assert!(into_ordered_path_exclusion(Vec::new(), &scope, &candidates).is_empty());
}
