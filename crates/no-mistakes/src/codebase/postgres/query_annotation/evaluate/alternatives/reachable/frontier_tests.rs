use super::*;
use crate::codebase::postgres::query_annotation::evaluate::scope::frontier_fixture;

fn collect(
    scopes: &[Scope],
    roots: RootSeeds<'_>,
    objects: &FxHashMap<u64, Vec<Value>>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Reachable {
    collect_from_roots(
        scopes,
        roots,
        &[],
        Arena {
            objects,
            extras: &Default::default(),
        },
        mapped,
        captured,
    )
}

#[test]
fn moved_frontiers_still_follow_selected_module_owners() {
    let (owner, scope) = frontier_fixture::load("small");
    let modules = FxHashMap::from_iter([(owner, 1), ("unused.mts".into(), 2)]);
    let scopes = vec![
        scope,
        Scope::from(FxHashMap::from_iter([("heap".into(), Value::Array(42))])),
        Scope::from(FxHashMap::from_iter([("unused".into(), Value::Array(99))])),
    ];
    let objects = FxHashMap::from_iter([(42, vec![Value::Unknown]), (99, vec![Value::Unknown])]);
    for (roots, environments, arguments) in [
        (RootSeeds::new(&modules), vec![0, 1], vec![42]),
        (
            RootSeeds::all_modules(&modules),
            vec![0, 1, 2],
            vec![42, 99],
        ),
    ] {
        let found = collect(
            &scopes,
            RootSeeds {
                environments: &[0],
                ..roots
            },
            &objects,
            &Default::default(),
            &Default::default(),
        );
        assert_eq!(found.environments, FxHashSet::from_iter(environments));
        assert_eq!(found.arguments, FxHashSet::from_iter(arguments));
    }
    assert!(scopes[0].is_module_only());
}

#[test]
fn frontier_walks_keep_dynamic_captured_and_mapped_edges_and_match_full_scan() {
    let (owner, scope) = frontier_fixture::load("small");
    let modules = FxHashMap::from_iter([(owner, 0)]);
    let mut scopes = vec![
        scope,
        Scope::from(FxHashMap::from_iter([(
            "heap".into(),
            Value::Arguments(77),
        )])),
    ];
    let objects = FxHashMap::from_iter([
        (42, vec![Value::Unknown]),
        (77, vec![Value::Prefix("SELECT 1".into(), true, Some(9))]),
    ]);
    // Registering these edges does not mutate the scope's immutable values.
    let mapped = FxHashMap::from_iter([(0, vec![(42, vec!["slot".into()])])]);
    let captured = FxHashMap::from_iter([(0, FxHashMap::from_iter([("origin".into(), 1)]))]);
    let found = collect(
        &scopes,
        RootSeeds::all_modules(&modules),
        &objects,
        &mapped,
        &captured,
    );
    assert_eq!(found.environments, FxHashSet::from_iter([0, 1]));
    assert_eq!(found.arguments, FxHashSet::from_iter([42, 77]));
    assert_eq!(found.identities, FxHashSet::from_iter([9, 42, 77]));
    assert!(scopes[0].is_module_only());
    scopes[0] = Scope::from((*scopes[0]).clone());
    let oracle = collect(
        &scopes,
        RootSeeds::all_modules(&modules),
        &objects,
        &mapped,
        &captured,
    );
    assert_eq!(found.environments, oracle.environments);
    assert_eq!(found.arguments, oracle.arguments);
    assert_eq!(found.identities, oracle.identities);
}

#[test]
fn empty_builder_indexes_never_hide_lexical_or_heap_references() {
    let (owner, template) = frontier_fixture::load("small");
    let Value::Function(function, _, _) = template["read0"].clone() else {
        panic!("saved function")
    };
    let scopes = vec![
        Scope::from(FxHashMap::from_iter([(
            "lexical".into(),
            Value::Function(
                function,
                owner.clone(),
                super::super::super::Capture::Lexical(1usize.into()),
            ),
        )])),
        Scope::from(FxHashMap::from_iter([(
            "arguments".into(),
            Value::Array(42),
        )])),
    ];
    for scope in &scopes {
        assert!(!scope.contains_builders(&FxHashSet::from_iter([42])));
        assert!(!scope.is_module_only());
    }
    let modules = FxHashMap::from_iter([(owner, 0)]);
    let objects = FxHashMap::from_iter([(42, vec![Value::Unknown])]);
    let found = collect(
        &scopes,
        RootSeeds::all_modules(&modules),
        &objects,
        &Default::default(),
        &Default::default(),
    );
    assert_eq!(found.environments, FxHashSet::from_iter([0, 1]));
    assert_eq!(found.arguments, FxHashSet::from_iter([42]));
}

#[test]
fn compaction_preserves_module_only_proofs_while_moving_frames() {
    let (owner, scope) = frontier_fixture::load("small");
    let mut scopes = vec![Scope::default(), scope];
    let mut modules = FxHashMap::from_iter([(owner.clone(), 1)]);
    let indices = super::super::frames::compact_modules(
        &mut scopes,
        &mut modules,
        &mut Default::default(),
        super::super::arena::ArenaMut {
            objects: &mut Default::default(),
            extras: &mut Default::default(),
        },
        &mut Default::default(),
        &mut Default::default(),
    );
    assert_eq!(indices.indices[&1], 0);
    assert_eq!(modules[&owner], 0);
    assert_eq!(scopes.len(), 1);
    assert!(scopes[0].is_module_only());
    let found = collect(
        &scopes,
        RootSeeds::new(&modules),
        &Default::default(),
        &Default::default(),
        &Default::default(),
    );
    assert!(
        found.environments.is_empty(),
        "selected roots are not implicitly all modules"
    );
}
