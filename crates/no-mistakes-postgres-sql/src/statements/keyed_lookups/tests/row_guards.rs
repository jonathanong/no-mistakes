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

#[test]
fn respects_planner_demand_and_equivalent_set_targets() {
    let facts = fixture_facts("review-planner-guards");
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();

    assert_eq!(
        sites,
        [
            (15, 16),
            (23, 24),
            (35, 36),
            (41, 42),
            (67, 68),
            (79, 80),
            (101, 104),
            (111, 113),
            (122, 124),
            (139, 140),
            (147, 148),
            (159, 161),
            (166, 167),
            (173, 174),
        ]
    );
}

#[test]
fn respects_output_stages_and_row_demand() {
    let facts = fixture_facts("review-output-stages");
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();

    assert_eq!(
        sites,
        [
            (7, 8),
            (18, 19),
            (24, 25),
            (40, 41),
            (50, 52),
            (61, 62),
            (66, 67),
            (77, 78),
            (83, 84),
            (92, 93),
            (97, 98),
            (106, 107),
            (111, 112),
        ]
    );
}

#[test]
fn preserves_returning_join_coalesce_cte_and_decimal_limit_guards() {
    let facts = fixture_facts("review-guard-propagation");
    let sites: Vec<_> = facts
        .duplicate_keyed_lookups
        .iter()
        .map(|fact| (fact.first_line, fact.line))
        .collect();

    assert_eq!(
        sites,
        [
            (9, 10),
            (15, 16),
            (21, 22),
            (35, 36),
            (42, 43),
            (54, 55),
            (61, 62),
            (68, 69),
            (75, 76),
            (82, 83),
            (102, 103),
            (112, 113),
            (130, 131),
            (139, 140),
            (178, 179),
            (186, 187),
            (194, 195),
        ]
    );
}
