use super::{
    hooks,
    tests::{evaluator, same_state},
    Evaluator, Value,
};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
type TestEvaluator<'a> = Evaluator<'a, fn(&str, &Path) -> Option<PathBuf>>;
fn spec() -> serde_json::Value {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("borrowed-builder-roots.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn legacy(evaluator: &mut TestEvaluator<'_>) {
    // Exercise the real unchanged snapshot clone path, not a modeled clone count.
    let roots = evaluator.live_builder_values();
    evaluator.untrusted_array_intrinsics = true;
    let (ids, _) = super::tests::collect_old(evaluator, &roots);
    evaluator.invalidate_builder_ids(ids);
}
fn compare(old: &mut TestEvaluator<'_>, new: &mut TestEvaluator<'_>) {
    let (_, old_clones) = hooks::measure(|| legacy(old));
    let (_, new_clones) = hooks::measure(|| new.invalidate_live_builders());
    same_state(old, new);
    assert!(old_clones > 0);
    assert_eq!(new_clones, 0);
    assert!(new.untrusted_array_intrinsics);
    eprintln!("borrowed live roots: actual root clones {old_clones} -> {new_clones}");
}
#[test]
fn borrowed_roots_match_legacy_for_every_wrapper_and_mutable_argument_cycle() {
    let spec = spec();
    let builder = |key: &str| {
        Value::Prefix(
            spec["prefix"].as_str().unwrap().into(),
            true,
            Some(spec[key].as_u64().unwrap()),
        )
    };
    let labels = spec["labels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|label| label.as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let argument = spec["argument"].as_u64().unwrap();
    let values: super::super::value::Values =
        vec![builder("builder"), Value::Arguments(argument)].into();
    let roots = vec![
        Value::Aggregate(values.clone()),
        Value::References(values.clone()),
        Value::Object(values.clone()),
        Value::Record(values.clone(), labels),
        Value::Joined(values.clone()),
        Value::Possible(values),
        Value::Promise(Box::new(builder("builder"))),
        Value::Evaluated(Box::new(builder("builder")), false),
        builder("builder"),
        Value::Array(argument),
    ];
    let scope = roots
        .into_iter()
        .enumerate()
        .map(|(i, value)| (i.to_string(), value))
        .collect::<FxHashMap<_, _>>();
    let files = FxHashMap::default();
    let mut old = evaluator(&files, scope.clone());
    let mut new = evaluator(&files, scope);
    for evaluator in [&mut old, &mut new] {
        evaluator.scopes.push(evaluator.scopes[0].clone());
        let names = FxHashSet::from_iter(["parameter".to_owned()]);
        evaluator.fresh_mapped_parameters.insert(0, names.clone());
        evaluator
            .fresh_mapped_argument_bindings
            .insert(argument, FxHashMap::from_iter([(0, names)]));
        evaluator.invalidated_builders.insert(999);

        evaluator
            .scopes
            .push(FxHashMap::from_iter([("unrelated".into(), Value::Number(4))]).into());
        evaluator.argument_objects.insert(
            argument,
            vec![
                Value::Array(argument),
                builder("dead_builder"),
                builder("builder"),
            ],
        );
        evaluator.argument_extra_slots.insert(
            argument,
            std::collections::BTreeMap::from_iter([(9, builder("extra_builder"))]),
        );
        evaluator
            .definite_deleted_argument_slots
            .insert((argument, 1));
        evaluator.deleted_argument_slots.insert((argument, Some(9)));
        evaluator
            .builder_updates
            .insert(spec["builder"].as_u64().unwrap(), builder("builder"));
    }
    let untouched = new.scopes[2].clone();
    let snapshot = new.scopes[0].clone();
    compare(&mut old, &mut new);
    assert!(new.fresh_mapped_parameters.is_empty());
    assert!(new.fresh_mapped_argument_bindings.is_empty());
    assert!(new.invalidated_builders.contains(&999));
    assert!(new.scopes[2].same_bindings(&untouched));
    assert!(!new.scopes[0].same_bindings(&snapshot));
    assert!(matches!(snapshot["8"], Value::Prefix(_, _, Some(_))));
    assert!(new.scopes[0]["8"] == Value::Unknown);
    assert!(!new
        .invalidated_builders
        .contains(&spec["dead_builder"].as_u64().unwrap()));
    assert!(new
        .invalidated_builders
        .contains(&spec["extra_builder"].as_u64().unwrap()));
    // Re-read live mutable slots after deletion is reversed: never memoize across calls.
    for evaluator in [&mut old, &mut new] {
        evaluator
            .definite_deleted_argument_slots
            .remove(&(argument, 1));
    }
    compare(&mut old, &mut new);
    assert!(new
        .invalidated_builders
        .contains(&spec["dead_builder"].as_u64().unwrap()));
}
#[test]
fn no_builder_and_empty_scopes_still_poison_intrinsics_without_cow() {
    let files = FxHashMap::default();
    let text = spec()["prefix"].as_str().unwrap().to_owned();
    let scope = FxHashMap::from_iter([("plain".into(), Value::Prefix(text, true, None))]);
    let mut old = evaluator(&files, scope.clone());
    let mut new = evaluator(&files, scope);
    let snapshot = new.scopes[0].clone();
    compare(&mut old, &mut new);
    assert!(new.scopes[0].same_bindings(&snapshot));
    old.scopes.clear();
    new.scopes.clear();
    old.untrusted_array_intrinsics = false;
    new.untrusted_array_intrinsics = false;
    let (_, old_clones) = hooks::measure(|| legacy(&mut old));
    let (_, new_clones) = hooks::measure(|| new.invalidate_live_builders());
    same_state(&old, &new);
    assert_eq!((old_clones, new_clones), (0, 0));
    assert!(new.untrusted_array_intrinsics);
}
