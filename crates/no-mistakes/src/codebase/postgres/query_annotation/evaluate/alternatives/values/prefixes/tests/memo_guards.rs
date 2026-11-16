use super::{fixture, legacy, Value};
use crate::codebase::postgres::query_annotation::evaluate::alternatives::values::unchanged;

#[test]
fn immutable_binding_memos_skip_walks_but_match_frozen_prefix_maps() {
    for name in ["all_container_kinds", "last_shared_write"] {
        let case = fixture::load(name);
        let snapshot = case.root.clone();
        assert!(unchanged(&case.root, &snapshot), "{name}");
        let before = legacy::collect(&case.root, case.arena(), &case.definite);
        let after = legacy::collect(&snapshot, case.arena(), &case.definite);
        assert_eq!(before, after, "{name}");
    }
}

#[test]
fn deep_shared_immutable_memo_matches_the_saved_prefix_map() {
    let case = fixture::load("shared_prefix_depth30");
    assert!(unchanged(&case.root, &case.root.clone()));
    let actual =
        super::super::collect::<false>(&case.root, case.arena(), &case.definite, &mut None);
    let owned = actual
        .into_iter()
        .map(|(id, (text, complete))| (id, (text.to_owned(), complete)))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(owned, case.expected);
}

#[test]
fn mutable_argument_cycles_sparse_slots_and_changed_arenas_never_skip() {
    for name in [
        "arena_before",
        "shared_arguments_depth30",
        "reentrant_record",
        "sparse_deleted_slots",
        "argument_writes_not_replayed",
    ] {
        let case = fixture::load(name);
        assert!(!unchanged(&case.root, &case.root.clone()), "{name}");
    }
    let before = fixture::load("arena_before");
    let after = fixture::load("arena_after");
    assert_ne!(
        legacy::collect(&before.root, before.arena(), &before.definite),
        legacy::collect(&before.root, after.arena(), &after.definite),
    );
}

#[test]
fn changed_and_new_argument_snapshots_invalidate_the_immutable_memo() {
    let case = fixture::load("all_container_kinds");
    assert!(unchanged(&case.root, &case.root.clone()));
    let mut changed = case.root.clone();
    let Value::Record(values, _) = &mut changed else {
        panic!("saved record");
    };
    values.push(Value::Arguments(1));
    assert!(!unchanged(&case.root, &changed));
    assert!(!unchanged(&changed, &changed.clone()));
    assert!(unchanged(&case.root, &case.root.clone()));
    let Value::Record(values, _) = &mut changed else {
        unreachable!()
    };
    values.pop();
    assert!(unchanged(&changed, &changed.clone()));
    assert!(!unchanged(&case.root, &changed));
}

#[test]
fn direct_prefixes_and_wrappers_require_exact_contents() {
    let prefix = Value::Prefix("before".into(), true, Some(1));
    let changed = Value::Prefix("after".into(), true, Some(1));
    assert!(unchanged(&prefix, &prefix.clone()));
    assert!(!unchanged(&prefix, &changed));
    let prefix = Value::Promise(Box::new(Value::Evaluated(Box::new(prefix), false)));
    assert!(unchanged(&prefix, &prefix.clone()));
    assert!(!unchanged(&prefix, &changed));
    assert!(unchanged(&Value::Unknown, &changed));
}

#[test]
fn every_immutable_container_kind_uses_its_shared_snapshot() {
    let case = fixture::load("all_container_kinds");
    let Value::Record(values, _) = case.root else {
        panic!("saved record");
    };
    for value in [
        Value::Aggregate(values.clone()),
        Value::References(values.clone()),
        Value::Object(values.clone()),
        Value::Record(values.clone(), Vec::new()),
        Value::Joined(values.clone()),
        Value::Possible(values),
    ] {
        assert!(unchanged(&value, &value.clone()));
        assert!(unchanged(&value, &Value::Unknown));
    }
}

#[test]
fn reference_free_replacements_match_the_raw_empty_intersection_for_mutable_roots() {
    use crate::codebase::postgres::query_annotation::Function;
    use crate::fx::FxHashSet;
    let function = Value::Function(
        std::sync::Arc::new(Function {
            start: 0,
            params: Vec::new(),
            body: Vec::new(),
            supported: true,
            asynchronous: false,
            arrow: true,
            self_name: None,
        }),
        std::path::PathBuf::from("callback.mts").into(),
        0,
    );
    for name in [
        "arena_before",
        "reentrant_record",
        "sparse_deleted_slots",
        "all_container_kinds",
    ] {
        let case = fixture::load(name);
        for after in [
            Value::Unknown,
            Value::Primitive,
            Value::Prefix("unidentified".into(), true, None),
            function.clone(),
            Value::Object(vec![Value::Unknown].into()),
            Value::Promise(Box::new(Value::Unknown)),
        ] {
            let mut changed = FxHashSet::default();
            super::super::super::changes(
                &case.root,
                &after,
                case.arena(),
                case.arena(),
                &mut changed,
                &case.definite,
                &mut None,
            );
            assert!(changed.is_empty(), "{name}");
            assert!(unchanged(&case.root, &after), "{name}");
        }
    }
}
