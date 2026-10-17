use super::*;
use crate::codebase::postgres::query_annotation::{
    evaluate::{value::Values, Capture},
    Function,
};
use std::sync::Arc;

fn function(capture: Capture) -> Value {
    Value::Function(
        Arc::new(Function {
            start: 0,
            params: vec![],
            body: vec![],
            supported: true,
            asynchronous: false,
            arrow: false,
            self_name: None,
        }),
        "helper.mts".into(),
        capture,
    )
}

fn compact_scope(scopes: &mut Vec<Scope>, env: Environment, preserved: Environment) -> Compaction {
    compact_from_roots(
        scopes,
        Roots {
            modules: &FxHashMap::from_iter([("helper.mts".into(), env)]),
            module_policy: reachable::ModulePolicy::Referenced,
            environments: &[env],
            arguments: &[],
            preserved,
            transient: &[],
        },
        &mut [],
        &mut Default::default(),
        ArenaMut {
            objects: &mut Default::default(),
            extras: &mut Default::default(),
        },
        &mut Default::default(),
        &mut Default::default(),
    )
}

#[test]
fn module_only_scope_stays_shared_when_its_owner_frame_moves() {
    for callback in [
        function(Capture::Module),
        Value::Object(vec![function(Capture::Module)].into()),
    ] {
        let initial = Scope::from(FxHashMap::from_iter([
            ("callback".into(), callback),
            (
                "builder".into(),
                Value::Prefix("SELECT 1".into(), true, Some(7)),
            ),
        ]));
        assert!(initial.contains_builders(&FxHashSet::from_iter([7])));
        let mut scopes = vec![Scope::default(), initial.clone()];
        let moved = compact_scope(&mut scopes, 1, 0);
        assert_eq!(moved.indices, FxHashMap::from_iter([(1, 0)]));
        // Mutable map access alone would detach a cached initial scope even
        // though Module values resolve their owner through the remapped catalog.
        assert!(std::ptr::eq(&*initial, &*scopes[0]));
        assert!(scopes[0].builder_index_initialized());
        assert_eq!(moved.identities, FxHashSet::from_iter([7]));
    }
}

#[test]
fn unchanged_lexical_owner_keeps_shared_scope_during_another_frame_move() {
    let initial = Scope::from(FxHashMap::from_iter([(
        "callback".into(),
        function(0usize.into()),
    )]));
    let mut scopes = vec![Scope::default(), Scope::default(), initial.clone()];
    let moved = compact_scope(&mut scopes, 2, 1);
    assert_eq!(moved.indices, FxHashMap::from_iter([(2, 1)]));
    assert!(std::ptr::eq(&*initial, &*scopes[1]));
}

#[test]
fn changed_lexical_owner_detaches_only_the_written_scope() {
    let initial = Scope::from(FxHashMap::from_iter([(
        "callback".into(),
        function(1usize.into()),
    )]));
    let mut scopes = vec![Scope::default(), initial.clone()];
    let moved = compact_scope(&mut scopes, 1, 0);
    assert_eq!(moved.indices, FxHashMap::from_iter([(1, 0)]));
    assert!(!std::ptr::eq(&*initial, &*scopes[0]));
    assert!(initial["callback"] == function(1usize.into()));
    assert!(scopes[0]["callback"] == function(0usize.into()));
}

#[test]
fn read_only_remap_predicate_follows_lexical_captures_through_containers() {
    let indices = FxHashMap::from_iter([(7, 2)]);
    let callback = function(7usize.into());
    let values = Values::from(vec![callback.clone()]);
    for value in [
        callback.clone(),
        Value::Promise(Box::new(Value::Evaluated(Box::new(callback), false))),
        Value::Aggregate(values.clone()),
        Value::References(values.clone()),
        Value::Object(values.clone()),
        Value::Record(values.clone(), vec!["callback".into()]),
        Value::Joined(values.clone()),
        Value::Possible(values),
    ] {
        assert!(remap::changes_value_environment(&value, &indices));
        assert!(!remap::changes_value_environment(
            &value,
            &FxHashMap::from_iter([(7, 7)])
        ));
    }
    for value in [
        Value::Unknown,
        function(Capture::Module),
        function(9usize.into()),
        Value::Object(vec![Value::Unknown].into()),
        Value::Object(vec![function(Capture::Module)].into()),
        Value::Object(vec![function(9usize.into())].into()),
    ] {
        assert!(!remap::changes_value_environment(&value, &indices));
    }
}
