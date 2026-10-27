use super::*;

#[test]
fn later_frames_changing_earlier_sources_preserve_original_one_pass_order() {
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    // Derive the real hash iteration order; reversing it would alter this pass.
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes = vec![
            scope(&[("x", Value::Number(9))]),
            scope(&[("x", Value::Number(1))]),
            scope(&[("x", Value::Number(2))]),
        ];
        capture(evaluator, 1, &[("x", 0)]);
        capture(evaluator, 2, &[("x", 0)]);
        let order = evaluator
            .captured_bindings
            .keys()
            .copied()
            .collect::<Vec<_>>();
        let (first, later) = (order[0], order[1]);
        capture(evaluator, first, &[("x", later)]);
        capture(evaluator, later, &[("x", 0)]);
    }
    assert_eq!(compare(&mut old, &mut new), 2);
    assert!(
        compare(&mut old, &mut new) > 0,
        "a later changed origin invalidates the earlier frame's memo"
    );
    assert_eq!(compare(&mut old, &mut new), 0);
}
