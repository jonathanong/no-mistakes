use super::*;

#[test]
fn origin_and_frame_mutations_and_replaced_allocations_force_refresh() {
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes = vec![scope(&[("x", Value::Number(1))]), Scope::default()];
        capture(evaluator, 1, &[("x", 0)]);
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert_eq!(compare(&mut old, &mut new), 0);
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes[0].insert("x".into(), Value::Number(2));
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert!(new.scopes[1]["x"] == Value::Number(2));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes[1].remove("x");
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert!(new.scopes[1]["x"] == Value::Number(2));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes[0] = scope(&[("x", Value::Number(3))]);
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert!(new.scopes[1]["x"] == Value::Number(3));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes.pop();
        evaluator.scopes.push(scope(&[("x", Value::Number(4))]));
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert!(new.scopes[1]["x"] == Value::Number(3));
}

#[test]
fn remapped_capture_sources_invalidate_memos_without_scope_changes() {
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes = vec![
            scope(&[("x", Value::Number(1))]),
            Scope::default(),
            scope(&[("x", Value::Number(2))]),
        ];
        capture(evaluator, 1, &[("x", 0)]);
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert_eq!(compare(&mut old, &mut new), 0);
    for evaluator in [&mut old, &mut new] {
        evaluator
            .captured_bindings
            .get_mut(&1)
            .unwrap()
            .remap(&FxHashMap::from_iter([(0, 2)]));
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert!(new.scopes[1]["x"] == Value::Number(2));
}

#[test]
fn deleting_an_origin_name_materializes_unknown_in_the_retained_frame() {
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes = vec![scope(&[("x", Value::Number(1))]), Scope::default()];
        capture(evaluator, 1, &[("x", 0)]);
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes[0].remove("x");
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert!(new.scopes[1]["x"] == Value::Unknown);
    assert_eq!(compare(&mut old, &mut new), 0);
}
