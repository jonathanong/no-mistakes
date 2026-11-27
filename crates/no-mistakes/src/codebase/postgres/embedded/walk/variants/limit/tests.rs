use super::{binary_alternatives, bounded, Recovered};
use crate::test_support::allocations::observe;

fn value(line: u32) -> Recovered {
    let mut value = Recovered::empty(line);
    value.sql = "/* lookup */ SELECT id FROM widgets".into();
    value.origins = vec![line; value.sql.len()];
    value
}

#[test]
fn variant_comparison_does_not_allocate() {
    let left = value(1);
    let mut right = left.clone();
    right.first_append_offset = Some(2);
    let (matches, allocations) = observe(|| left.same_value(&right));
    assert!(matches);
    assert_eq!(allocations, 0);

    right.origins[0] += 1;
    assert!(!left.same_value(&right));
}

#[test]
fn distinct_variants_move_into_the_result() {
    let input = (1..=16).map(value).collect();
    let (variants, allocations) = observe(|| bounded(input));
    let variants = variants.unwrap();
    assert!(variants.iter().map(|value| value.line).eq(1..=16));
    assert!(allocations <= 4, "{allocations} allocations");
}

#[test]
fn duplicate_variants_merge_append_metadata() {
    let mut first = value(1);
    first.choices = vec![(9, 1), (3, 0)];
    first.first_append_offset = Some(2);
    first.append_sites = vec![31];
    first.decision_offs = vec![(9, 10, 0)];

    let mut duplicate = first.clone();
    duplicate.choices = vec![(3, 0), (9, 1), (3, 0)];
    duplicate.first_append_offset = Some(4);
    duplicate.append_sites = vec![32];
    duplicate.decision_offs = vec![(3, 12, 1)];
    duplicate.type_annotation = true;

    let variants = bounded(vec![first, duplicate]).unwrap();
    let [merged] = variants.as_slice() else {
        panic!("expected one deduplicated variant, got {}", variants.len());
    };
    assert_eq!(merged.choices, [(3, 0), (9, 1)]);
    assert_eq!(merged.first_append_offset, Some(4));
    assert_eq!(merged.append_sites, [31, 32]);
    assert_eq!(merged.append_decisions, [3, 9]);
    assert!(merged.type_annotation);
}

#[test]
fn binary_join_ignores_only_the_joined_choice_and_enumeration_marker() {
    let mut left = value(1);
    left.choices = vec![(2, 0), (1, 0)];
    let mut right = left.clone();
    right.choices = vec![(2, 1), (1, 0), (1, 0)];
    right.enumerated = true;
    let mut distinct = right.clone();
    distinct.origins[0] += 1;
    let left = vec![left];
    let right = vec![distinct.clone(), right];
    let (variants, allocations) = observe(|| binary_alternatives(left, right, 2));
    let variants = variants.unwrap();
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[0].choices, [(1, 0)]);
    assert!(variants[0].enumerated);
    assert_eq!(variants[1].choices, [(1, 0), (2, 1)]);
    assert_eq!(variants[1].origins, distinct.origins);
    assert!(allocations <= 4, "{allocations} allocations");
}
