use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;

fn callback(env: usize) -> Value {
    Value::Function(
        tests::saved_function(),
        std::path::PathBuf::from("reads.mts").into(),
        env,
    )
}

fn parity(
    scopes: &[Scope],
    base: usize,
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
    prefix: &mut Prefix,
) -> Reachable {
    let roots = (0..base).collect::<Vec<_>>();
    let seeds = || RootSeeds {
        environments: &roots,
        ..Default::default()
    };
    let legacy = collect_from_roots(scopes, seeds(), &[], arena, mapped, captured);
    let cached = collect_cached(
        scopes,
        seeds(),
        &[],
        arena,
        mapped,
        captured,
        (prefix, base),
    );
    assert_eq!(legacy.environments, cached.environments);
    assert_eq!(legacy.arguments, cached.arguments);
    assert_eq!(legacy.identities, cached.identities);
    cached
}

#[test]
fn removing_auxiliary_roots_drops_dead_cycles_without_changing_scope_bindings() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let scopes = vec![
        Scope::default(),
        FxHashMap::from_iter([
            ("next".into(), callback(2)),
            ("args".into(), Value::Arguments(7)),
        ])
        .into(),
        FxHashMap::from_iter([("back".into(), callback(1))]).into(),
    ];
    let original = scopes[0].clone();
    let objects = ArgumentObjects::from_iter([
        (7, vec![Value::Arguments(7)]),
        (8, vec![Value::Arguments(8)]),
    ]);
    let arena = Arena {
        objects: &objects,
        extras: &Default::default(),
    };
    let mut captured =
        FxHashMap::from_iter([(0, FxHashMap::from_iter([("source".into(), 1)]).into())]);
    let mut mapped = FxHashMap::from_iter([(0, vec![(8, vec!["mapped".into()])])]);
    let mut prefix = Prefix::default();
    let live = parity(&scopes, 1, arena, &mapped, &captured, &mut prefix);
    assert_eq!(live.environments, FxHashSet::from_iter([0, 1, 2]));
    assert_eq!(live.arguments, FxHashSet::from_iter([7, 8]));
    assert_eq!(live.identities, FxHashSet::from_iter([7, 8]));
    // Neither auxiliary map is part of the Scope Arc or its immutable memo.
    captured.clear();
    mapped.clear();
    assert!(original.same_bindings(&scopes[0]));
    let dead = parity(&scopes, 1, arena, &mapped, &captured, &mut prefix);
    assert_eq!(dead.environments, FxHashSet::from_iter([0]));
    assert!(dead.arguments.is_empty());
    assert!(dead.identities.is_empty());
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.prefix_scope_scans"], 3);
    assert_eq!(work["postgres.annotation.prefix_binding_scans"], 3);
    assert_eq!(work["postgres.annotation.reachability_walks"], 4);
}

#[test]
fn duplicate_roots_across_preserved_scopes_keep_exact_counts_until_last_removal() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let value = Value::Possible(
        vec![
            callback(2),
            Value::Arguments(7),
            Value::Prefix("SELECT 1".into(), true, Some(19)),
        ]
        .into(),
    );
    let first = Scope::from(FxHashMap::from_iter([("root".into(), value)]));
    // Distinct preserved frames contribute two references even with shared bindings.
    let mut scopes = vec![
        first.clone(),
        first,
        FxHashMap::from_iter([("cycle".into(), callback(2))]).into(),
    ];
    let objects = ArgumentObjects::from_iter([
        (
            7,
            vec![
                Value::Array(8),
                Value::Prefix("SELECT 2".into(), true, Some(20)),
            ],
        ),
        (8, vec![Value::Arguments(8)]),
    ]);
    let arena = Arena {
        objects: &objects,
        extras: &Default::default(),
    };
    let mut prefix = Prefix::default();
    for removed in 0..=2 {
        if removed > 0 {
            scopes[removed - 1].clear();
        }
        let found = parity(
            &scopes,
            2,
            arena,
            &Default::default(),
            &Default::default(),
            &mut prefix,
        );
        if removed < 2 {
            assert_eq!(found.environments, FxHashSet::from_iter([0, 1, 2]));
            assert_eq!(found.arguments, FxHashSet::from_iter([7, 8]));
            assert_eq!(found.identities, FxHashSet::from_iter([7, 8, 19, 20]));
        } else {
            assert_eq!(found.environments, FxHashSet::from_iter([0, 1]));
            assert!(found.arguments.is_empty());
            assert!(found.identities.is_empty());
        }
    }
    let work = observer.snapshot().work;
    // The first two frames share one immutable binding allocation. Each clear
    // computes a fresh empty summary; the private callback frame is scanned once.
    assert_eq!(work["postgres.annotation.prefix_scope_scans"], 4);
    assert_eq!(work["postgres.annotation.prefix_binding_scans"], 2);
    assert_eq!(work["postgres.annotation.reachability_walks"], 6);
}
