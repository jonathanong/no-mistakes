use super::*;

#[test]
fn fingerprint_collisions_still_compare_exact_candidate_values() {
    let old = (0, ValueExpression::Unknown);
    let new = (1, ValueExpression::Unknown);
    let mut candidates = Candidates::default();
    // Force a colliding bucket so equality, rather than the fingerprint, decides.
    candidates.items.push(old);
    candidates
        .buckets
        .insert(super::super::hash::fingerprint(&new), vec![0]);
    assert!(!candidates.contains(&new));
    candidates.insert(new.clone());
    candidates.insert(new.clone());
    assert!(candidates.contains(&new));
    assert_eq!(candidates.items.len(), 2);
}
