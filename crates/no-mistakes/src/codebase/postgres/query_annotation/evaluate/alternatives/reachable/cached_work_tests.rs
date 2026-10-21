use super::*;

#[test]
fn immutable_scope_scan_work_is_initial_plus_changed_bindings() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
    let mut scopes = (0..128)
        .map(|_| {
            Scope::from(FxHashMap::from_iter(
                (0..64).map(|n| (format!("global{n}"), Value::Primitive)),
            ))
        })
        .collect::<Vec<_>>();
    let mut cache = Prefix::default();
    let roots = (0..scopes.len()).collect::<Vec<_>>();
    for arm in 0..16 {
        if arm == 8 {
            scopes[0].insert("new".into(), Value::Unknown);
        }
        let found = collect_cached(
            &scopes,
            RootSeeds {
                environments: &roots,
                ..Default::default()
            },
            &[],
            Arena {
                objects: &Default::default(),
                extras: &Default::default(),
            },
            &Default::default(),
            &Default::default(),
            (&mut cache, scopes.len()),
        );
        assert_eq!(found.environments.len(), 128);
    }
    let work = observer.snapshot().work;
    assert_eq!(work["postgres.annotation.prefix_scope_scans"], 129);
    assert_eq!(
        work["postgres.annotation.prefix_binding_scans"],
        128 * 64 + 65
    );
}

#[test]
fn direct_edges_match_full_value_visitor_for_all_variants_and_shared_dag() {
    let leaf = Value::Possible(
        vec![
            callback(3),
            Value::Array(8),
            Value::Prefix("SELECT 1".into(), true, Some(9)),
        ]
        .into(),
    );
    let mut nested = leaf;
    for _ in 0..12 {
        nested = Value::Aggregate(vec![nested.clone(), nested].into());
    }
    let nested = Value::Joined(vec![nested].into());
    let nested = Value::Record(vec![nested].into(), vec!["a".into()]);
    let nested = Value::Object(vec![nested].into());
    let nested = Value::References(vec![nested].into());
    let values = vec![
        Value::Promise(Box::new(Value::Evaluated(Box::new(nested), true))),
        Value::Arguments(7),
        Value::Number(1),
        Value::Boolean(true),
        Value::Failure("test"),
        Value::Unknown,
        Value::Primitive,
        Value::Unsupported,
        Value::SlotDeletion,
        Value::Prefix("plain".into(), true, None),
    ];
    let mut legacy = Reachable::default();
    for value in &values {
        legacy.value(value);
    }
    let direct =
        crate::codebase::postgres::query_annotation::evaluate::references::Edges::collect(&values);
    assert_eq!(
        direct.environments,
        FxHashSet::from_iter(legacy.pending_env)
    );
    assert_eq!(direct.arguments, FxHashSet::from_iter(legacy.pending_args));
    assert_eq!(direct.identities, legacy.identities);
}
