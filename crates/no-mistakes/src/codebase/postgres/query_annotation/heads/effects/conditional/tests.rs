use super::{merge, Conditional};

#[test]
fn overlapping_groups_require_every_effect_only_at_the_shared_call() {
    let mut conditions = vec![Conditional {
        spans: vec![(1, 2), (3, 4)],
        required: vec![(11, 12)],
    }];
    merge(
        &mut conditions,
        vec![Conditional {
            spans: vec![(3, 4), (5, 6)],
            required: vec![(13, 14)],
        }],
    );
    assert_eq!(conditions.len(), 3);
    assert_eq!(conditions[0].spans, vec![(1, 2)]);
    assert_eq!(conditions[0].required, vec![(11, 12)]);
    assert_eq!(conditions[1].spans, vec![(3, 4)]);
    assert_eq!(conditions[1].required, vec![(11, 12), (13, 14)]);
    assert_eq!(conditions[2].spans, vec![(5, 6)]);
    assert_eq!(conditions[2].required, vec![(13, 14)]);
}
