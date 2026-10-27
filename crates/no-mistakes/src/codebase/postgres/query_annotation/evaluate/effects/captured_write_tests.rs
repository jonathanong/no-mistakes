use super::{
    tests::{evaluator, same_state},
    Evaluator, Value,
};
use crate::codebase::postgres::query_annotation::{Expr, Function, Step};
use crate::fx::{FxHashMap, FxHashSet};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
type TestEvaluator<'a> = Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>>;

fn prefix(id: Option<u64>) -> Value {
    Value::Prefix("SELECT 1".into(), false, id)
}
fn function(arrow: bool) -> Function {
    Function {
        start: 0,
        params: vec!["parameter".into()],
        body: vec![
            Step::Bind("bound".into(), Expr::Unknown),
            Step::Hoisted("hoisted".into(), Expr::Unknown),
            Step::Var("variable".into()),
            Step::Reserve(vec!["reserved".into()]),
        ],
        supported: true,
        asynchronous: false,
        arrow,
        self_name: Some("self".into()),
    }
}
fn legacy(evaluator: &mut TestEvaluator<'_>, function: &Function) {
    let mut shadowed = function.params.iter().cloned().collect::<FxHashSet<_>>();
    shadowed.extend(function.self_name.iter().cloned());
    if !function.arrow {
        shadowed.insert("arguments".into());
    }
    for step in &function.body {
        match step {
            Step::Bind(name, _) | Step::Hoisted(name, _) | Step::Var(name) => {
                shadowed.insert(name.clone());
            }
            Step::Reserve(names) => shadowed.extend(names.iter().cloned()),
            _ => {}
        }
    }
    let values = evaluator.scopes[0]
        .iter()
        .filter(|(name, _)| !shadowed.contains(*name))
        .map(|(_, value)| value.clone())
        .collect::<Vec<_>>();
    evaluator.invalidate_builders(&values);
    // Keep the original unconditional mutable access as the physical-work oracle.
    for (name, value) in &mut evaluator.scopes[0] {
        if !shadowed.contains(name) && matches!(value, Value::Prefix(_, _, _)) {
            *value = Value::Unknown;
        }
    }
}

#[test]
fn nested_captured_values_keep_warm_scope_identity_and_match_original_full_state() {
    let callback = Value::Function(Arc::new(function(true)), "callback.mts".into(), 0);
    let scope = FxHashMap::from_iter([
        (
            "nested".into(),
            Value::Record(vec![prefix(Some(7)), Value::Array(1)].into(), vec![]),
        ),
        ("callback".into(), callback),
        ("parameter".into(), prefix(None)),
    ]);
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files, scope.clone()), evaluator(&files, scope));
    for evaluator in [&mut old, &mut new] {
        evaluator
            .argument_objects
            .insert(1, vec![Value::Arguments(1), prefix(Some(8))]);
        let names = FxHashSet::from_iter(["mapped".into()]);
        evaluator.fresh_mapped_parameters.insert(0, names.clone());
        evaluator
            .fresh_mapped_argument_bindings
            .insert(1, FxHashMap::from_iter([(0, names)]));
    }
    let original = old.scopes[0].clone();
    let snapshot = new.scopes[0].clone();
    let references = new.scopes[0].references() as *const _;
    for _ in 0..2 {
        legacy(&mut old, &function(true));
        new.invalidate_captured(0, &function(true));
        same_state(&old, &new);
        assert!(new.scopes[0].same_bindings(&snapshot));
        assert_eq!(
            new.scopes[0].cached_references().unwrap() as *const _,
            references
        );
    }
    assert!(!old.scopes[0].same_bindings(&original));
    assert!(new.scopes[0]["parameter"] == prefix(None));
    assert!(new.fresh_mapped_parameters.is_empty());
    assert_eq!(new.invalidated_builders, FxHashSet::from_iter([1, 7, 8]));
    assert!(
        !new.untrusted_array_intrinsics,
        "captured invalidation does not poison intrinsics"
    );
}

#[test]
fn direct_identified_and_unidentified_prefixes_and_all_shadow_sources_match_original() {
    for arrow in [false, true] {
        let mut scope = FxHashMap::from_iter([
            ("identified".into(), prefix(Some(7))),
            ("unidentified".into(), prefix(None)),
            ("arguments".into(), prefix(None)),
            ("alias".into(), prefix(Some(9))),
            ("nested".into(), Value::Object(vec![prefix(Some(9))].into())),
        ]);
        for name in [
            "parameter",
            "self",
            "bound",
            "hoisted",
            "variable",
            "reserved",
        ] {
            scope.insert(name.into(), prefix(None));
        }
        // A shadowed identified alias remains subject to global ID invalidation.
        scope.insert("self".into(), prefix(Some(9)));
        let files = FxHashMap::default();
        let (mut old, mut new) = (evaluator(&files, scope.clone()), evaluator(&files, scope));
        let snapshot = new.scopes[0].clone();
        new.scopes[0].references();
        legacy(&mut old, &function(arrow));
        new.invalidate_captured(0, &function(arrow));
        same_state(&old, &new);
        assert!(!new.scopes[0].same_bindings(&snapshot));
        assert!(new.scopes[0].cached_references().is_none());
        for name in ["identified", "unidentified", "alias", "self"] {
            assert!(new.scopes[0][name] == Value::Unknown, "{name}");
        }
        for name in ["parameter", "bound", "hoisted", "variable", "reserved"] {
            assert!(new.scopes[0][name] == prefix(None), "{name}");
        }
        assert!(new.scopes[0]["arguments"] == if arrow { Value::Unknown } else { prefix(None) });
    }
}
#[test]
fn capture_noop_retains_shared_warm_argument_free_comparison_preconditions() {
    let scope = FxHashMap::from_iter([
        ("nested".into(), Value::Object(vec![prefix(Some(7))].into())),
        ("parameter".into(), prefix(None)),
    ]);
    let files = FxHashMap::default();
    let (mut old, mut new) = (evaluator(&files, scope.clone()), evaluator(&files, scope));
    let original = old.scopes[0].clone();
    let snapshot = new.scopes[0].clone();
    let references = new.scopes[0].references() as *const _;
    assert!(new.scopes[0]
        .cached_references()
        .unwrap()
        .arguments
        .is_empty());
    assert!(new.scopes[0].contains_reference());
    legacy(&mut old, &function(true));
    new.invalidate_captured(0, &function(true));
    same_state(&old, &new);
    // Both comparison shortcuts require this same Arc; prefix comparison also
    // requires a still-warm summary proving that mutable arguments are absent.
    assert!(new.scopes[0].same_bindings(&snapshot));
    assert_eq!(
        new.scopes[0].cached_references().unwrap() as *const _,
        references
    );
    assert!(new.scopes[0]
        .cached_references()
        .unwrap()
        .arguments
        .is_empty());
    assert!(old.scopes[0].cached_references().is_none());
    assert!(!old.scopes[0].same_bindings(&original));
}
