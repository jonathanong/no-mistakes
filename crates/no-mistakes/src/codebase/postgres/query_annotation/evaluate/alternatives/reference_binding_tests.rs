use super::super::{ArgumentObjects, Value};
use crate::codebase::postgres::query_annotation::evaluate::Scope;
use crate::fx::{FxHashMap, FxHashSet};

#[test]
fn indexed_prefix_comparisons_match_the_legacy_full_binding_scan() {
    let prefix = |text: &str, id| Value::Prefix(text.into(), true, Some(id));
    let before = Scope::from(FxHashMap::from_iter([
        ("scalar".into(), Value::Unknown),
        ("direct".into(), prefix("SELECT old", 1)),
        ("dropped".into(), prefix("SELECT missing", 2)),
        ("args".into(), Value::Arguments(7)),
        ("array".into(), Value::Array(7)),
        (
            "wrapped".into(),
            Value::Promise(Box::new(Value::Evaluated(
                Box::new(Value::Possible(vec![prefix("SELECT old", 3)].into())),
                true,
            ))),
        ),
    ]));
    let mut after = before.clone();
    after.insert("scalar".into(), prefix("SELECT introduced", 9));
    after.insert("direct".into(), prefix("SELECT changed", 1));
    after.remove("dropped");
    let before_objects =
        ArgumentObjects::from_iter([(7, vec![Value::Arguments(7), prefix("SELECT old", 4)])]);
    let after_objects =
        ArgumentObjects::from_iter([(7, vec![Value::Arguments(7), prefix("SELECT changed", 4)])]);
    let extras = FxHashMap::default();
    let definite = FxHashSet::default();
    let mut work = None;
    let mut legacy = FxHashSet::default();
    let mut indexed = FxHashSet::default();
    let mut compare = |bindings: Vec<(&String, &Value)>, changed: &mut FxHashSet<u64>| {
        for (name, value) in bindings {
            super::values::changes(
                value,
                super::bindings::binding_or_unproven(&after, name),
                super::arena::Arena {
                    objects: &before_objects,
                    extras: &extras,
                },
                super::arena::Arena {
                    objects: &after_objects,
                    extras: &extras,
                },
                changed,
                &definite,
                &mut work,
            );
        }
    };
    // Keep the full old loop, including scalar work, as the semantic oracle.
    compare(before.iter().collect(), &mut legacy);
    compare(before.reference_bindings().collect(), &mut indexed);
    assert_eq!(indexed, legacy);
    assert_eq!(indexed, FxHashSet::from_iter([1, 4]));
}
