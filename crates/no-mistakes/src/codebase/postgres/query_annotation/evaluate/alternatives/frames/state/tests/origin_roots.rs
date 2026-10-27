use super::*;
use crate::codebase::postgres::query_annotation::evaluate::alternatives::arena::Arena;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;

#[test]
fn complete_environment_roots_omit_capture_names_with_exact_reachability() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("capture-prefix-duplicate-origins.json");
    let saved: Vec<(String, usize)> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let captured =
        CapturedBindings::from_iter([(0, saved.into_iter().map(|(name, _)| (name, 1)).collect())]);
    let scopes: Vec<Scope> = vec![
        Scope::default(),
        FxHashMap::from_iter([("private arguments".into(), Value::Arguments(7))]).into(),
    ];
    let objects = ArgumentObjects::from_iter([(
        7,
        vec![Value::Prefix(
            "/* reached */ SELECT 1".into(),
            true,
            Some(19),
        )],
    )]);
    let extras = FxHashMap::default();
    let mapped = FxHashMap::default();
    let roots = [0, 1];
    let read = |captures| {
        reachable::collect_warm_from_roots(
            &scopes,
            reachable::RootSeeds {
                environments: &roots,
                ..Default::default()
            },
            &[],
            Arena {
                objects: &objects,
                extras: &extras,
            },
            &mapped,
            captures,
        )
    };
    assert_eq!(captured[&0].max_origin(), Some(1));
    let counter = reachable::scan_hooks::OriginGuard::new();
    let old = read(&captured);
    assert_eq!(counter.visits(), 256);
    drop(counter);
    let empty = CapturedBindings::default();
    let selected = capture_edges_for_all_roots(&captured, scopes.len(), &empty);
    assert!(std::ptr::eq(selected, &empty));
    let counter = reachable::scan_hooks::OriginGuard::new();
    let new = read(selected);
    assert_eq!(counter.visits(), 0);
    assert_eq!(old.environments, new.environments);
    assert_eq!(old.arguments, new.arguments);
    assert_eq!(old.identities, new.identities);
    assert_eq!(new.environments, FxHashSet::from_iter([0, 1]));
    assert_eq!(new.arguments, FxHashSet::from_iter([7]));
    assert_eq!(new.identities, FxHashSet::from_iter([7, 19]));
}

#[test]
fn complete_root_guard_keeps_out_of_range_origins_and_owners_on_original_walk() {
    let empty = CapturedBindings::default();
    for captured in [
        CapturedBindings::from_iter([(0, FxHashMap::from_iter([("outside".into(), 2)]).into())]),
        CapturedBindings::from_iter([(
            2,
            FxHashMap::from_iter([("outside owner".into(), 0)]).into(),
        )]),
    ] {
        let selected = capture_edges_for_all_roots(&captured, 2, &empty);
        assert!(std::ptr::eq(selected, &captured));
    }
    // A private environment is not redundant for an arbitrary-root caller.
    // The shared collect API must retain that edge even after this optimization.
    let captured =
        CapturedBindings::from_iter([(0, FxHashMap::from_iter([("private".into(), 1)]).into())]);
    let scopes = [Scope::default(), Scope::default()];
    let objects = ArgumentObjects::default();
    let extras = FxHashMap::default();
    let found = reachable::collect_warm_from_roots(
        &scopes,
        reachable::RootSeeds {
            environments: &[0],
            ..Default::default()
        },
        &[],
        Arena {
            objects: &objects,
            extras: &extras,
        },
        &FxHashMap::default(),
        &captured,
    );
    assert_eq!(found.environments, FxHashSet::from_iter([0, 1]));
}
