use super::super::super::Value;
use super::{binding_or_unproven, join};
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::fx_map;

fn prefix(text: &str, id: Option<u64>) -> Value {
    Value::Prefix(text.to_string(), true, id)
}

fn scope(entries: &[(&str, Value)]) -> Scope {
    let mut values = fx_map();
    for (name, value) in entries {
        values.insert((*name).to_string(), value.clone());
    }
    values.into()
}

#[test]
fn missing_alternative_bindings_stay_unproven() {
    let annotated = prefix("/* left */ SELECT 1", Some(1));
    let other = prefix("/* right */ SELECT 2", Some(1));
    let stable = prefix("/* stable */ SELECT 1", Some(3));
    let original = prefix("/* original */ SELECT 1", Some(4));
    let rewritten = prefix("/* rewritten */ SELECT 1", Some(4));
    let mut joined = vec![scope(&[
        ("onlyLeft", annotated.clone()),
        ("created", annotated.clone()),
        ("agreed", annotated.clone()),
        ("stable", stable.clone()),
        ("kept", rewritten.clone()),
        ("mismatched", annotated.clone()),
        ("scalar", prefix("SELECT 1", None)),
        ("diverged", Value::Primitive),
    ])];
    let current = vec![scope(&[
        ("created", other.clone()),
        ("agreed", annotated.clone()),
        ("stable", stable.clone()),
        ("kept", original.clone()),
        ("mismatched", other.clone()),
        ("scalar", prefix("SELECT 2", None)),
        ("diverged", Value::Unsupported),
        ("afterOnly", annotated.clone()),
    ])];
    let initial = vec![scope(&[
        (
            "stable",
            prefix("/* different initial */ SELECT 9", Some(3)),
        ),
        ("kept", original.clone()),
        ("mismatched", prefix("/* other id */ SELECT 1", Some(8))),
        ("scalar", Value::Unknown),
        ("diverged", Value::Primitive),
    ])];
    join(&mut joined, &current, &initial);
    for name in ["onlyLeft", "created", "agreed"] {
        assert!(
            joined[0][name] == Value::Unknown,
            "{name} must not keep a proven annotation"
        );
    }
    assert!(!joined[0].contains_key("afterOnly"));
    assert!(joined[0]["stable"] == stable);
    assert!(joined[0]["kept"] == original);
    assert!(joined[0]["mismatched"] == Value::Aggregate(vec![annotated, other]));
    assert!(joined[0]["scalar"] == Value::Unknown);
    assert!(joined[0]["diverged"] == Value::Aggregate(vec![Value::Primitive, Value::Unsupported]));

    let present = scope(&[("present", Value::Primitive)]);
    assert!(binding_or_unproven(&present, "present") == &Value::Primitive);
    assert!(binding_or_unproven(&present, "missing") == &Value::Unknown);
    let mut changed = crate::fx::FxHashSet::default();
    super::super::values::changes(
        &prefix("/* kept */ SELECT 1", Some(5)),
        binding_or_unproven(&present, "missing"),
        super::super::arena::Arena {
            objects: &Default::default(),
            extras: &Default::default(),
        },
        super::super::arena::Arena {
            objects: &Default::default(),
            extras: &Default::default(),
        },
        &mut changed,
        &Default::default(),
    );
    assert!(
        changed.is_empty(),
        "a dropped binding is a rebind, not a mutated builder"
    );
}
