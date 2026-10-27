use super::*;
use crate::codebase::postgres::query_annotation::evaluate::Value;
use std::cell::Cell;

thread_local! { static READS: Cell<usize> = const { Cell::new(0) }; }
pub(super) fn record_binding_reads() {
    READS.set(READS.get() + 2);
}
fn proofs(names: &[&str]) -> Proofs {
    FxHashMap::from_iter([(0, names.iter().map(|name| (*name).into()).collect())])
}
fn scope(values: &[(&str, Value)]) -> Scope {
    values
        .iter()
        .map(|(name, value)| ((*name).into(), value.clone()))
        .collect::<FxHashMap<_, _>>()
        .into()
}
fn compare(
    initial: Proofs,
    current: &Proofs,
    before: &[Scope],
    after: &[Scope],
) -> (Proofs, usize, usize) {
    let mut old = initial.clone();
    let mut reads = 0;
    for (env, names) in &mut old {
        names.retain(|name| {
            current.get(env).is_some_and(|fresh| fresh.contains(name)) && {
                reads += 2;
                before[*env].get(name) == after[*env].get(name)
            }
        });
    }
    let mut new = initial;
    READS.set(0);
    unchanged(&mut new, current, before, after);
    assert_eq!(
        new, old,
        "the entire proof table, including empty sets, is unchanged"
    );
    (new, reads, READS.get())
}

#[test]
fn shared_scopes_still_drop_nonfresh_names_and_keep_missing_bindings() {
    let before = scope(&[
        ("retained", Value::Number(1)),
        ("dropped", Value::Number(2)),
    ]);
    let after = before.clone();
    let (result, old, new) = compare(
        proofs(&["retained", "dropped", "missing"]),
        &proofs(&["retained", "missing"]),
        &[before],
        &[after],
    );
    assert_eq!(result, proofs(&["retained", "missing"]));
    assert_eq!(
        (old, new),
        (4, 0),
        "shared immutable snapshots need no binding reads"
    );
}

#[test]
fn changed_scopes_preserve_value_equality_and_both_missing_semantics() {
    let before = scope(&[
        ("same", Value::Number(1)),
        ("changed", Value::Number(2)),
        ("removed", Value::Number(3)),
    ]);
    let mut after = before.clone();
    after.insert("changed".into(), Value::Number(9));
    after.remove("removed");
    assert!(!before.same_bindings(&after));
    let names = proofs(&["same", "changed", "removed", "missing"]);
    let (result, old, new) = compare(names.clone(), &names, &[before], &[after]);
    assert_eq!(result, proofs(&["same", "missing"]));
    assert_eq!((old, new), (8, 8));
}

#[test]
fn empty_or_nonfresh_proofs_do_not_index_invalid_environments() {
    let cases = [
        (
            FxHashMap::from_iter([(99, FxHashSet::default())]),
            Proofs::default(),
        ),
        (
            FxHashMap::from_iter([(99, FxHashSet::from_iter(["absent".into()]))]),
            Proofs::default(),
        ),
        (
            FxHashMap::from_iter([(99, FxHashSet::from_iter(["absent".into()]))]),
            FxHashMap::from_iter([(99, FxHashSet::from_iter(["other".into()]))]),
        ),
    ];
    for (initial, current) in cases {
        let (result, old, new) = compare(initial, &current, &[], &[]);
        assert!(result[&99].is_empty());
        assert_eq!((old, new), (0, 0));
    }
}

#[test]
fn eligible_names_still_check_scope_bounds() {
    for (before, after) in [(vec![], vec![]), (vec![Scope::default()], vec![])] {
        let result = std::panic::catch_unwind(|| {
            let mut joined = proofs(&["eligible"]);
            unchanged(&mut joined, &proofs(&["eligible"]), &before, &after);
        });
        assert!(result.is_err());
    }
}

#[test]
fn mutable_argument_contents_do_not_change_immutable_binding_equality() {
    let before = scope(&[(
        "nested",
        Value::Object(vec![Value::Arguments(1), Value::Array(2)].into()),
    )]);
    let after = before.clone();
    // This helper compares Value identities, not the mutable argument arenas.
    let names = proofs(&["nested"]);
    let (result, old, new) = compare(
        names.clone(),
        &names,
        std::slice::from_ref(&before),
        &[after],
    );
    assert_eq!(result, names);
    assert_eq!((old, new), (2, 0));
    let mut detached = before.clone();
    detached.insert("other".into(), Value::Unknown);
    let (_, old, new) = compare(names.clone(), &names, &[before], &[detached]);
    assert_eq!(
        (old, new),
        (2, 2),
        "a COW write invalidates the pointer shortcut"
    );
}
