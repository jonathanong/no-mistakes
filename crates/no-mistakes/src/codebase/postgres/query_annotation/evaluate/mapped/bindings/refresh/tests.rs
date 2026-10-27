use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Scope;
mod fixture;
use fixture::{capture, compare, evaluator, scope};
mod invalidation;
mod ordering;

#[test]
fn warm_refresh_skips_names_and_missing_bindings_still_materialize_unknown() {
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes = vec![scope(&[("x", Value::Number(1))]), Scope::default()];
        capture(evaluator, 1, &[("x", 0), ("missing", 0)]);
    }
    assert_eq!(compare(&mut old, &mut new), 2);
    assert!(new.scopes[1]["missing"] == Value::Unknown);
    let snapshot = new.scopes[1].clone();
    assert_eq!(compare(&mut old, &mut new), 0);
    assert!(new.scopes[1].same_bindings(&snapshot));
}

#[test]
fn frame_as_origin_missing_names_record_post_write_epochs() {
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes = vec![Scope::default()];
        capture(evaluator, 0, &[("missing", 0)]);
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    assert!(new.scopes[0]["missing"] == Value::Unknown);
    assert_eq!(compare(&mut old, &mut new), 0);
}

#[test]
fn empty_capture_ownership_never_indexes_invalid_frame_or_origin() {
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    capture(&mut old, 99, &[]);
    capture(&mut new, 99, &[]);
    assert_eq!(compare(&mut old, &mut new), 0);
}

#[test]
fn mutable_arguments_do_not_invalidate_a_refresh_that_only_copies_value_ids() {
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files), evaluator(&files));
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes = vec![scope(&[("arg", Value::Arguments(7))]), Scope::default()];
        evaluator.argument_objects.insert(7, vec![Value::Number(1)]);
        capture(evaluator, 1, &[("arg", 0)]);
    }
    assert_eq!(compare(&mut old, &mut new), 1);
    for evaluator in [&mut old, &mut new] {
        evaluator.argument_objects.get_mut(&7).unwrap()[0] = Value::Number(2);
    }
    assert_eq!(compare(&mut old, &mut new), 0);
    assert!(new.scopes[1]["arg"] == Value::Arguments(7));
}

#[test]
fn invalid_nonempty_ownership_keeps_the_original_bounds_failure() {
    for (frame, origin) in [(99, 0), (0, 99)] {
        let result = std::panic::catch_unwind(|| {
            let files = FxHashMap::default();
            let mut evaluator = evaluator(&files);
            evaluator.scopes.push(Scope::default());
            capture(&mut evaluator, frame, &[("missing", origin)]);
            evaluator.refresh_captured_bindings();
        });
        assert!(result.is_err());
    }
}
