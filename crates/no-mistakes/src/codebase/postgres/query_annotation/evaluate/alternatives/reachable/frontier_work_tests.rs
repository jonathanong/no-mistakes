use super::*;
use crate::codebase::postgres::query_annotation::evaluate::scope::frontier_fixture;

fn work(name: &str, expected_bindings: u64) -> (u64, u64) {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let (owner, scope) = frontier_fixture::load(name);
        assert_eq!(scope.len() as u64, expected_bindings);
        assert!(scope.is_module_only());
        assert!(scope.contains_key("read0") && scope.contains_key("statement"));
        let scopes = [scope];
        let modules = FxHashMap::from_iter([(owner, 0)]);
        for _ in 0..8 {
            let found = collect_from_roots(
                &scopes,
                RootSeeds::all_modules(&modules),
                &[],
                Arena {
                    objects: &Default::default(),
                    extras: &Default::default(),
                },
                &Default::default(),
                &Default::default(),
            );
            assert_eq!(found.environments, FxHashSet::from_iter([0]));
            assert!(found.arguments.is_empty() && found.identities.is_empty());
        }
    });
    let counts = observer.snapshot().work;
    assert_eq!(counts["postgres.annotation.reachability_walks"], 8);
    // Cold template construction/validation remains visible once per binding.
    assert_eq!(
        counts["postgres.annotation.initial_template.cached_bindings"],
        expected_bindings
    );
    (
        counts
            .get("postgres.annotation.reachability_binding_visits")
            .copied()
            .unwrap_or(0),
        counts["postgres.annotation.initial_template.template_builds"],
    )
}

#[test]
fn saved_inert_frontiers_skip_repeated_actual_binding_walks() {
    let small = work("small", 5);
    let large = work("large", 129);
    assert_eq!(
        (small, large),
        ((0, 1), (0, 1)),
        "the old collector visits 5 and 129 bindings on each of eight walks"
    );
}

#[test]
fn selected_roots_and_invalidated_frontiers_still_count_actual_binding_visits() {
    let observer = crate::diagnostics::InvocationObserver::new(true);
    crate::diagnostics::with_observer(Some(observer.clone()), || {
        let (owner, mut scope) = frontier_fixture::load("small");
        let modules = FxHashMap::from_iter([(owner, 0)]);
        for referenced in [true, false] {
            if !referenced {
                // Even a no-op mutable view revokes the proof conservatively.
                scope.entry("statement".into()).or_insert(Value::Unknown);
                assert!(!scope.is_module_only());
            }
            let roots = if referenced {
                RootSeeds::new(&modules)
            } else {
                RootSeeds::all_modules(&modules)
            };
            collect_from_roots(
                &[scope.clone()],
                RootSeeds {
                    environments: &[0],
                    ..roots
                },
                &[],
                Arena {
                    objects: &Default::default(),
                    extras: &Default::default(),
                },
                &Default::default(),
                &Default::default(),
            );
        }
    });
    assert_eq!(
        observer.snapshot().work["postgres.annotation.reachability_binding_visits"],
        10
    );
}
