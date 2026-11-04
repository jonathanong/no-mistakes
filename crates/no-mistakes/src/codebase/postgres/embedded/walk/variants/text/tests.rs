use super::Recovered;

#[test]
fn equivalent_alternatives_keep_only_shared_append_proof() {
    let mut left = Recovered::empty(1);
    let mut right = left.clone();
    left.first_append_offset = Some(3);
    right.first_append_offset = Some(8);
    assert!(left.same_value(&right));
    left.merge_alternative(&right);
    assert_eq!(left.first_append_offset, Some(8));
    right.first_append_offset = None;
    left.merge_alternative(&right);
    assert_eq!(left.first_append_offset, None);
}
