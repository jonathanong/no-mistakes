use super::*;
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;

fn assert_parity(
    scopes: &[Scope],
    base: usize,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
    prefix: &mut Prefix,
) -> Reachable {
    let objects = ArgumentObjects::from_iter([
        (7, vec![Value::Arguments(8)]),
        (8, vec![Value::Arguments(7)]),
    ]);
    let arena = Arena {
        objects: &objects,
        extras: &Default::default(),
    };
    let roots = (0..base).collect::<Vec<_>>();
    let seeds = || RootSeeds {
        environments: &roots,
        ..Default::default()
    };
    let old = collect_from_roots(scopes, seeds(), &[], arena, mapped, captured);
    let (new, probes) = measured(|| {
        collect_cached(
            scopes,
            seeds(),
            &[],
            arena,
            mapped,
            captured,
            (prefix, base),
        )
    });
    assert_eq!(probes, mapped.len() + captured.len());
    assert!(probes < 2 * base);
    assert_eq!(old.environments, new.environments);
    assert_eq!(old.arguments, new.arguments);
    assert_eq!(old.identities, new.identities);
    new
}

#[test]
fn sparse_auxiliary_edges_are_live_across_writes_and_last_root_removal() {
    let base = 64;
    let scopes = (0..base + 2).map(|_| Scope::default()).collect::<Vec<_>>();
    let mut mapped = FxHashMap::from_iter([(0, vec![(7, vec![])])]);
    let mut captured = CapturedBindings::from_iter([
        (0, FxHashMap::from_iter([("first".into(), base)]).into()),
        (
            base,
            FxHashMap::from_iter([("next".into(), base + 1)]).into(),
        ),
        (
            base + 1,
            FxHashMap::from_iter([("cycle".into(), base)]).into(),
        ),
    ]);
    let mut prefix = Prefix::default();
    let first = assert_parity(&scopes, base, &mapped, &captured, &mut prefix);
    assert_eq!(first.environments.len(), base + 2);
    assert_eq!(first.arguments, FxHashSet::from_iter([7, 8]));
    mapped.clear();
    captured.remove(&0);
    // Unchanged Scope snapshots must not retain the now-unrooted private cycles.
    let dead = assert_parity(&scopes, base, &mapped, &captured, &mut prefix);
    assert_eq!(dead.environments.len(), base);
    assert!(dead.arguments.is_empty());
    assert!(dead.identities.is_empty());
    mapped.insert(1, vec![(8, vec![])]);
    captured.insert(
        1,
        FxHashMap::from_iter([("replacement".into(), base + 1)]).into(),
    );
    let restored = assert_parity(&scopes, base, &mapped, &captured, &mut prefix);
    assert_eq!(restored.environments.len(), base + 2);
    assert_eq!(restored.arguments, FxHashSet::from_iter([7, 8]));
}
