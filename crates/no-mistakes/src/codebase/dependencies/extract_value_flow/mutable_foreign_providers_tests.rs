use super::*;

#[test]
fn foreign_two_latest_distinct_owners_answer_every_suffix_query() {
    for entries in [
        vec![(1, 1), (2, 1)],
        vec![(1, 1), (2, 1), (3, 2), (4, 3), (5, 1)],
        vec![(5, 1), (4, 3), (3, 2), (2, 1), (1, 1)],
        vec![(3, 2), (1, 1), (5, 1), (2, 1), (4, 3)],
    ] {
        let mut providers = Providers {
            newest: (entries[0].0, CallableId(entries[0].1)),
            other: None,
        };
        for (id, owner) in entries.iter().skip(1) {
            providers.insert(*id, CallableId(*owner));
        }
        for owner in 0..=3 {
            for before in 0..=6 {
                assert_eq!(
                    providers.after(CallableId(owner), before),
                    entries
                        .iter()
                        .any(|(id, candidate)| *id >= before && *candidate != owner)
                );
            }
        }
    }
    assert!(size_of::<Providers>() <= 5 * size_of::<usize>());
}

#[test]
fn foreign_provider_index_without_a_retained_root_is_empty() {
    let origins = ReceiverBindingOrigins::default();
    let index = origins.foreign_provider_index(&(0, "missing".into()), &[], Vec::new());
    assert!(index.references(Some(CallableId(0)), 0).is_empty());
    assert!(origins
        .module_provider_index(&(0, "missing".into()), &[])
        .is_empty());
}

#[test]
fn callable_member_providers_are_available_without_a_root_rebinding() {
    let origins = ReceiverBindingOrigins::default();
    let reference = (
        ((0, "provider".into()), vec![Some("get".into())]),
        vec![Some((9, Some(CallableId(1))))],
        (9, Some(CallableId(1))),
    );
    let index = origins.foreign_provider_index(
        &(0, "missing".into()),
        &[],
        vec![(reference, CallableId(1))],
    );
    let selected = index.references(None, usize::MAX);
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].1, vec![None]);
    assert_eq!(selected[0].2, (0, None));
    assert_eq!(index.references(Some(CallableId(1)), 0).len(), 1);
    assert_eq!(index.references(Some(CallableId(2)), usize::MAX).len(), 1);
}
