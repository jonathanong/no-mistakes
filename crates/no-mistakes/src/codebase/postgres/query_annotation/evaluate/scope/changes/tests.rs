use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Value;
use crate::fx::FxHashMap;

fn scope() -> Scope {
    let mut values = (0..62)
        .map(|index| (format!("stable{index}"), Value::Primitive))
        .collect::<FxHashMap<_, _>>();
    values.insert("kept".into(), Value::Unknown);
    values.insert("changed".into(), Value::Primitive);
    values.into()
}
fn names(scope: &Scope, initial: &Scope) -> Vec<String> {
    let mut names = scope
        .changed_binding_names(initial)
        .unwrap()
        .into_iter()
        .collect::<Vec<_>>();
    names.sort();
    names
}
#[test]
fn cold_unique_writes_have_no_journal() {
    let mut scope = scope();
    scope.insert("fresh".into(), Value::Primitive);
    scope.remove("kept");
    *scope.get_mut("changed").unwrap() = Value::Unknown;
    assert!(scope.1 .0.is_none());
}
#[test]
fn sparse_writes_and_absent_keys_are_tracked_across_nested_snapshots() {
    let initial = scope();
    let mut first = initial.clone();
    first.insert("fresh".into(), Value::Primitive);
    first.insert("fresh".into(), Value::Primitive);
    let mut second = first.clone();
    second.remove("kept");
    *second.get_mut("changed").unwrap() = Value::Unknown;
    assert_eq!(names(&first, &initial), ["fresh"]);
    assert_eq!(names(&second, &first), ["changed", "kept"]);
    assert_eq!(names(&second, &initial), ["changed", "fresh", "kept"]);
    assert!(initial.contains_key("kept"));
}
#[test]
fn mutated_origin_identity_cannot_match_a_descendants_old_snapshot() {
    let mut initial = scope();
    let mut child = initial.clone();
    child.insert("fresh".into(), Value::Primitive);
    // Only a weak identity remains once the child detaches. make_mut must
    // dissociate it when the original is subsequently changed in place.
    initial.insert("other".into(), Value::Primitive);
    assert!(child.changed_binding_names(&initial).is_none());
}
#[test]
fn bulk_mutation_and_unrelated_scopes_fall_back() {
    let initial = scope();
    let mut changed = initial.clone();
    changed.entry("fresh".into()).or_insert(Value::Primitive);
    assert!(changed.changed_binding_names(&initial).is_none());
    assert!(scope().changed_binding_names(&initial).is_none());
}
#[test]
fn journal_depth_and_name_storage_are_bounded() {
    let initial = scope();
    let mut current = initial.clone();
    for index in 0..MAX_DEPTH + 2 {
        let snapshot = current.clone();
        current.insert(index.to_string(), Value::Primitive);
        assert_eq!(names(&current, &snapshot), [index.to_string()]);
    }
    assert!(current.changed_binding_names(&initial).is_none());
    let snapshot = current.clone();
    for index in 0..=MAX_NAMES {
        current.insert(format!("key{index}"), Value::Primitive);
    }
    assert!(current.changed_binding_names(&snapshot).is_none());
}
#[test]
fn histories_do_not_keep_values_or_old_binding_maps_alive() {
    let initial = scope();
    let origin = Arc::downgrade(&initial.0);
    let mut changed = initial.clone();
    changed.insert("fresh".into(), Value::Primitive);
    drop(initial);
    assert!(origin.upgrade().is_none());
    assert!(changed.1 .0.is_some());
}

#[test]
fn tiny_copy_on_write_scopes_keep_the_borrowed_scan() {
    let initial: Scope = FxHashMap::from_iter([("key".into(), Value::Primitive)]).into();
    let mut changed = initial.clone();
    changed.insert("key".into(), Value::Unknown);
    assert!(changed.1 .0.is_none());
    assert!(changed.changed_binding_names(&initial).is_none());
}

#[test]
fn keyed_forks_after_bulk_mutation_do_not_recover_old_ancestry() {
    let initial = scope();
    let mut changed = initial.clone();
    changed.entry("bulk".into()).or_insert(Value::Primitive);
    let anchor = changed.clone();
    changed.insert("fresh".into(), Value::Primitive);
    assert_eq!(names(&changed, &anchor), ["fresh"]);
    assert!(changed.changed_binding_names(&initial).is_none());
}

#[test]
fn evicted_histories_track_new_anchors_without_recovering_old_ones() {
    for name_limit in [false, true] {
        let initial = scope();
        let mut changed = initial.clone();
        if name_limit {
            for index in 0..=MAX_NAMES {
                changed.insert(format!("key{index}"), Value::Primitive);
            }
        } else {
            for index in 0..MAX_DEPTH + 2 {
                let _snapshot = changed.clone();
                changed.insert(index.to_string(), Value::Primitive);
            }
        }
        assert!(changed.changed_binding_names(&initial).is_none());
        let anchor = changed.clone();
        changed.insert("fresh".into(), Value::Primitive);
        assert_eq!(names(&changed, &anchor), ["fresh"]);
        assert!(changed.changed_binding_names(&initial).is_none());
    }
}

#[test]
fn duplicate_writes_at_the_name_limit_remain_conservative() {
    let initial = scope();
    let mut changed = initial.clone();
    for index in 0..MAX_NAMES {
        changed.insert(format!("key{index}"), Value::Primitive);
    }
    assert_eq!(changed.1 .0.as_ref().unwrap().names.len(), MAX_NAMES);
    changed.insert("key0".into(), Value::Unknown);
    assert!(changed.1 .0.is_none());
    assert!(changed.changed_binding_names(&initial).is_none());
    assert!(changed["key0"] == Value::Unknown);
}

#[test]
fn exact_admission_boundaries_and_shrinking_scopes_are_conservative() {
    let make = |count| {
        (0..count)
            .map(|index| (index.to_string(), Value::Primitive))
            .collect::<FxHashMap<_, _>>()
            .into()
    };
    let small: Scope = make(31);
    let mut changed = small.clone();
    changed.insert("fresh".into(), Value::Unknown);
    assert_eq!(changed.len(), 32);
    assert!(changed.1.is_empty(), "the 31-binding source uses the scan");
    let anchor = changed.clone();
    changed.insert("0".into(), Value::Unknown);
    assert_eq!(names(&changed, &anchor), ["0"]);
    assert!(changed.changed_binding_names(&small).is_none());

    let initial: Scope = make(32);
    let mut changed = initial.clone();
    changed.remove("0");
    assert_eq!(changed.len(), 31);
    assert!(
        !changed.1.is_empty(),
        "the 32-binding removal records its key"
    );
    assert!(
        changed.changed_binding_names(&initial).is_none(),
        "the resulting 31-binding join uses a scan"
    );
    *changed.get_mut("1").unwrap() = Value::Unknown;
    assert!(
        changed.1.is_empty(),
        "the next 31-binding write drops history"
    );
    assert!(changed.changed_binding_names(&initial).is_none());
}

#[test]
fn cumulative_names_prune_parents_before_depth_eviction() {
    let initial = scope();
    let mut changed = initial.clone();
    for arm in 0..32 {
        let anchor = changed.clone();
        for key in 0..8 {
            changed.insert(format!("arm{arm}-key{key}"), Value::Primitive);
        }
        assert_eq!(names(&changed, &anchor).len(), 8);
    }
    assert_eq!(changed.1 .0.as_ref().unwrap().retained_names, MAX_NAMES);
    let anchor = changed.clone();
    changed.insert("fresh".into(), Value::Primitive);
    let journal = changed.1 .0.as_ref().unwrap();
    assert!(journal.parent.is_none());
    assert_eq!(journal.depth, 0);
    assert_eq!(journal.retained_names, 1);
    assert_eq!(names(&changed, &anchor), ["fresh"]);
    assert!(changed.changed_binding_names(&initial).is_none());
}

#[test]
fn scope_equality_ignores_different_journal_histories() {
    let initial = scope();
    let mut changed = initial.clone();
    changed.insert("changed".into(), Value::Primitive);
    assert!(!changed.same_bindings(&initial));
    assert!(!changed.1.is_empty());
    assert!(changed == initial);
    changed.entry("changed".into()).or_insert(Value::Unknown);
    assert!(changed.1.is_empty());
    assert!(changed == initial);
}
