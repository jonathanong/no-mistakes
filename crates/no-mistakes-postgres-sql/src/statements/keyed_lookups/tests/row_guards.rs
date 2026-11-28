use super::fixture_facts;

#[test]
fn reports_only_reachable_row_lookups_and_distinct_frame_outputs() {
    let facts = fixture_facts("review-row-guards");
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();

    assert_eq!(
        sites,
        [
            (12, 13),
            (20, 21),
            (27, 28),
            (32, 33),
            (48, 49),
            (54, 55),
            (60, 61),
            (66, 67),
            (86, 87),
            (97, 96),
            (108, 109),
            (120, 121),
            (131, 132),
            (142, 143),
            (148, 149),
            (169, 170),
            (175, 176),
            (184, 185),
            (200, 201),
            (206, 207),
            (212, 213),
            (218, 219),
            (224, 225),
            (230, 231),
            (241, 242),
        ]
    );
}
