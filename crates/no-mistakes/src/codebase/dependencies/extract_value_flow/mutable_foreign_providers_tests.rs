use super::*;

#[test]
fn two_latest_distinct_owners_answer_every_suffix_query() {
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
