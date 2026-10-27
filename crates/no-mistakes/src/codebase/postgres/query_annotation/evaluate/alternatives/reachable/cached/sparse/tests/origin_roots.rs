use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;

fn compare(
    scopes: &[Scope],
    objects: &ArgumentObjects,
    captured: &CapturedBindings,
    prefix: &mut Prefix,
) -> (usize, usize, Reachable) {
    let roots = [0];
    let mapped = FxHashMap::default();
    let extras = FxHashMap::default();
    let read = |prefix: &mut Prefix| {
        super::super::super::collect_cached(
            scopes,
            RootSeeds {
                environments: &roots,
                ..Default::default()
            },
            &[],
            Arena {
                objects,
                extras: &extras,
            },
            &mapped,
            captured,
            (prefix, 1),
        )
    };
    let mode = Guard::new(true);
    ORIGIN_VISITS.with(|count| count.set(0));
    let old = read(&mut Prefix::default());
    let old_names = ORIGIN_VISITS.with(Cell::get);
    drop(mode);
    let _mode = Guard::new(false);
    ORIGIN_VISITS.with(|count| count.set(0));
    let new = read(prefix);
    let new_names = ORIGIN_VISITS.with(Cell::get);
    assert_eq!(old.environments, new.environments);
    assert_eq!(old.arguments, new.arguments);
    assert_eq!(old.identities, new.identities);
    (old_names, new_names, new)
}

#[test]
fn warmed_prefix_origins_skip_name_edges_but_revisit_remapped_and_private_origins() {
    let path = crate::test_support::rule_fixture_root("postgres-require-query-annotation")
        .join("capture-prefix-duplicate-origins.json");
    let saved: Vec<(String, usize)> =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(saved.len(), 256);
    let mut captured = CapturedBindings::from_iter([(0, saved.into_iter().collect())]);
    let scopes = vec![
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
    let mut prefix = Prefix::default();
    // Count the actual reachability name iterator after the shared lazy max
    // summary is warm; its one-time construction is tested by the cache owner.
    assert_eq!(captured[&0].max_origin(), Some(0));
    let (old_names, new_names, found) = compare(&scopes, &objects, &captured, &mut prefix);
    assert_eq!((old_names, new_names), (256, 0));
    assert_eq!(found.environments, FxHashSet::from_iter([0]));
    assert!(found.arguments.is_empty() && found.identities.is_empty());

    captured
        .get_mut(&0)
        .unwrap()
        .remap(&FxHashMap::from_iter([(0, 1)]));
    assert_eq!(captured[&0].max_origin(), Some(1));
    let (old_names, new_names, found) = compare(&scopes, &objects, &captured, &mut prefix);
    assert_eq!((old_names, new_names), (256, 256));
    assert_eq!(found.environments, FxHashSet::from_iter([0, 1]));
    assert_eq!(found.arguments, FxHashSet::from_iter([7]));
    assert_eq!(found.identities, FxHashSet::from_iter([7, 19]));

    captured
        .get_mut(&0)
        .unwrap()
        .remap(&FxHashMap::from_iter([(1, 0)]));
    assert_eq!(captured[&0].max_origin(), Some(0));
    assert_eq!(compare(&scopes, &objects, &captured, &mut prefix).1, 0);
    let mut mixed = (*captured[&0]).clone();
    mixed.insert("new private origin".into(), 1);
    captured.insert(0, mixed.into());
    assert_eq!(captured[&0].max_origin(), Some(1));
    let (old_names, new_names, found) = compare(&scopes, &objects, &captured, &mut prefix);
    assert_eq!((old_names, new_names), (257, 257));
    assert_eq!(found.identities, FxHashSet::from_iter([7, 19]));
}
