use super::*;

#[test]
fn collection_fails_closed_at_depth_and_state_boundaries() {
    let mut assigned = BTreeMap::new();
    let mut combinations = Vec::new();
    let mut remaining = 1;
    assert!(!collect_combinations(
        &[],
        &[],
        super::super::MAX_STATIC_MATRIX_AXIS_DEPTH + 1,
        &mut assigned,
        &mut combinations,
        &mut remaining,
    ));

    let axes = vec![(
        "target".to_string(),
        vec![Value::String("linux".to_string())],
    )];
    let mut remaining = 1;
    assert!(!collect_combinations(
        &axes,
        &[],
        0,
        &mut assigned,
        &mut combinations,
        &mut remaining,
    ));
}
