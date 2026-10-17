use super::super::Capture;
use super::*;

#[test]
fn frontier_reuses_the_existing_binding_and_scope_layout() {
    struct LegacyBindings {
        _values: FxHashMap<String, Value>,
        _builders: OnceLock<FxHashSet<u64>>,
    }
    assert_eq!(
        std::mem::size_of::<Bindings>(),
        std::mem::size_of::<LegacyBindings>()
    );
    assert_eq!(
        std::mem::align_of::<Bindings>(),
        std::mem::align_of::<LegacyBindings>()
    );
    assert_eq!(
        std::mem::size_of::<Scope>(),
        std::mem::size_of::<Arc<Bindings>>()
    );
}

#[test]
fn frontier_builder_queries_keep_the_proof_and_semantic_equality() {
    let (_, scope) = super::frontier_fixture::load("small");
    assert!(scope.is_module_only());
    assert!(!scope.contains_builders(&FxHashSet::from_iter([17, 42])));
    assert!(scope.is_module_only());
    let plain = Scope::from((*scope).clone());
    assert!(scope == plain);
    assert!(!plain.contains_builders(&FxHashSet::from_iter([17])));
    assert!(
        !plain.is_module_only(),
        "an empty builder index is not a frontier proof"
    );
}

#[test]
fn frontier_whitelist_checks_complete_values_not_just_builder_membership() {
    let (owner, saved) = super::frontier_fixture::load("small");
    let Value::Function(function, _, _) = saved["read0"].clone() else {
        panic!("saved function")
    };
    let mut scalar = Scope::from(FxHashMap::from_iter([
        ("unknown".into(), Value::Unknown),
        ("unsupported".into(), Value::Unsupported),
        ("number".into(), Value::Number(1)),
        ("boolean".into(), Value::Boolean(true)),
        ("text".into(), Value::Prefix("SELECT 1".into(), false, None)),
    ]));
    // A prior empty builder index must not prevent installing a stronger proof.
    assert!(!scalar.contains_builders(&FxHashSet::from_iter([1])));
    scalar.mark_module_only();
    assert!(scalar.is_module_only());
    for value in [
        Value::Arguments(1),
        Value::Array(1),
        Value::Prefix("SELECT 1".into(), true, Some(1)),
        Value::Function(function, owner, Capture::Lexical(0usize.into())),
        Value::Promise(Box::new(Value::Unknown)),
        Value::Evaluated(Box::new(Value::Unknown), true),
        Value::Aggregate(vec![Value::Unknown].into()),
        Value::Failure("unproven"),
    ] {
        let mut scope = Scope::from(FxHashMap::from_iter([("value".into(), value)]));
        scope.mark_module_only();
        assert!(!scope.is_module_only());
    }
}

#[test]
fn every_mutable_scope_view_invalidates_shared_and_unique_frontiers() {
    let (_, original) = super::frontier_fixture::load("small");
    for operation in 0..8 {
        let mut scope = original.clone();
        match operation {
            0 => {
                scope.insert("extra".into(), Value::Unknown);
            }
            1 => {
                scope.entry("statement".into()).or_insert(Value::Unknown);
            }
            2 => {
                let _ = scope.get_mut("statement");
            }
            3 => {
                scope.remove("absent");
            }
            4 => {
                let _ = scope.values_mut();
            }
            5 => {
                let _ = scope.iter_mut();
            }
            6 => {
                let _ = (&mut scope).into_iter();
            }
            _ => {
                scope.retain(|_, _| true);
            }
        }
        assert!(!scope.is_module_only());
        assert!(original.is_module_only());
        assert!(!std::ptr::eq(&*scope, &*original));
        scope.insert(
            "builder".into(),
            Value::Prefix("SELECT 1".into(), true, Some(42)),
        );
        assert!(scope.contains_builders(&FxHashSet::from_iter([42])));
    }
    let mut unique = original.clone();
    drop(original);
    unique.insert(
        "builder".into(),
        Value::Prefix("SELECT 1".into(), true, Some(42)),
    );
    assert!(!unique.is_module_only());
    assert!(unique.contains_builders(&FxHashSet::from_iter([42])));
}

#[test]
fn full_scope_replacement_restores_the_matching_proof_and_values() {
    let (_, original) = super::frontier_fixture::load("small");
    let mut replaced = original.clone();
    replaced.insert("statement".into(), Value::Array(42));
    assert!(!replaced.is_module_only());
    replaced.clone_from(&original);
    assert!(replaced.is_module_only());
    assert!(replaced == original);
    let mut prefix = [Scope::default()];
    prefix.clone_from_slice(&[original]);
    assert!(prefix[0].is_module_only());
}
