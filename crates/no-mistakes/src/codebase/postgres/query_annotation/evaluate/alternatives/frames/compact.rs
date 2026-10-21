use super::*;

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn compact(
    scopes: &mut Vec<Scope>,
    cache: ModuleRoots<'_>,
    returned: &mut [Value],
    mapped: &mut FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    arena: ArenaMut<'_>,
    fresh: &mut FxHashMap<Environment, FxHashSet<String>>,
    captured: &mut CapturedBindings,
) -> Compaction {
    let original = cache.original;
    let mut roots = (0..original).collect::<Vec<_>>();
    roots.extend(cache.modules.values().copied());
    let mut argument_roots = Vec::new();
    for initial in cache.initials.iter() {
        roots.extend(initial.roots());
        argument_roots.extend(initial.arguments());
    }
    let found = reachable::collect_cached(
        scopes,
        reachable::RootSeeds {
            environments: &roots,
            arguments: &argument_roots,
            transient: cache.transient,
        },
        returned,
        arena.read(),
        mapped,
        captured,
        (&mut *cache.prefix, original),
    );
    let indices = compact_collected(
        scopes,
        original,
        returned,
        mapped,
        arena,
        fresh,
        Collected {
            captured,
            reachable: found,
        },
    );
    if indices.indices.iter().any(|(old, new)| old != new) {
        *cache.prefix = reachable::Prefix::default();
    }
    for initial in cache.initials.iter_mut() {
        initial.remap(&indices.indices);
    }
    for env in cache.modules.values_mut() {
        if let Some(remapped) = indices.indices.get(env) {
            *env = *remapped;
        }
    }
    indices
}
