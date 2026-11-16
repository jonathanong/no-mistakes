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
    let mut memoized = FxHashSet::default();
    let mut compare =
        |bindings: Vec<(&String, &Value)>, changed: &mut FxHashSet<u64>, memoized: bool| {
            let mut walks = 0;
            for (name, value) in bindings {
                let after = super::bindings::binding_or_unproven(&after, name);
                if memoized && super::values::unchanged(value, after) {
                    continue;
                }
                walks += 1;
                super::values::changes(
                    value,
                    after,
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
            walks
        };
    // Keep the full old loop, including scalar work, as the semantic oracle.
    let legacy_walks = compare(before.iter().collect(), &mut legacy, false);
    let indexed_walks = compare(before.reference_bindings().collect(), &mut indexed, false);
    let memoized_walks = compare(before.reference_bindings().collect(), &mut memoized, true);
    assert!(memoized_walks < indexed_walks && indexed_walks < legacy_walks);
    assert_eq!(memoized, legacy);
    assert_eq!(indexed, legacy);
    assert_eq!(indexed, FxHashSet::from_iter([1, 4]));
}
