use super::*;

mod sparse;

pub(in crate::codebase::postgres::query_annotation::evaluate::alternatives) fn collect_cached(
    scopes: &[Scope],
    roots: RootSeeds<'_>,
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
    cached: (&mut Prefix, usize),
) -> Reachable {
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment("postgres.annotation.reachability_walks", 1);
    }
    let (prefix, base) = cached;
    // The caller supplies every preserved environment as a root.
    prefix.refresh(&scopes[..base]);
    // Distinct frames often share one immutable snapshot. Union its edges once.
    let mut seen_edges = FxHashSet::<usize>::default();
    let mut found = Reachable::default();
    prefix.seed(&mut found);
    found
        .pending_env
        .extend(roots.environments[base..].iter().copied());
    found.pending_args.extend(roots.arguments.iter().copied());
    // Auxiliary maps are mutable independently of Scope snapshots. Always read
    // their current edges, including changes to preserved prefix environments.
    sparse::auxiliary_prefix(&mut found, base, mapped, captured);
    for root in roots.transient {
        match root {
            TransientRoot::Argument(id) => found.pending_args.push(*id),
            TransientRoot::Builder(id) => {
                found.identities.insert(*id);
            }
            TransientRoot::Environment(env) => found.pending_env.push(*env),
        }
    }
    for value in returned {
        found.value(value);
    }
    while !found.pending_env.is_empty() || !found.pending_args.is_empty() {
        while let Some(env) = found.pending_env.pop() {
            if found.environments.insert(env) {
                auxiliary(&mut found, env, mapped, captured);
                let edges = scopes[env].references();
                if seen_edges.insert(std::ptr::from_ref(edges) as usize) {
                    found.pending_env.extend(edges.environments.iter().copied());
                    found.pending_args.extend(edges.arguments.iter().copied());
                    found.identities.extend(edges.identities.iter().copied());
                }
            }
        }
        while let Some(id) = found.pending_args.pop() {
            if found.arguments.insert(id) {
                found.identities.insert(id);
                // Dense snapshots cache direct edges, never the mutable arena's
                // transitive closure. A write detaches and invalidates this memo.
                let edges = arena.objects[&id].references();
                if seen_edges.insert(std::ptr::from_ref(edges) as usize) {
                    found.pending_env.extend(edges.environments.iter().copied());
                    found.pending_args.extend(edges.arguments.iter().copied());
                    found.identities.extend(edges.identities.iter().copied());
                }
                for value in arena
                    .extras
                    .get(&id)
                    .into_iter()
                    .flat_map(|slots| slots.values())
                {
                    found.value(value);
                }
            }
        }
    }
    found
}

fn auxiliary(
    found: &mut Reachable,
    env: Environment,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
) {
    if let Some(origins) = captured.get(&env) {
        found.pending_env.extend(origins.values().copied());
    }
    if let Some(bindings) = mapped.get(&env) {
        found
            .pending_args
            .extend(bindings.iter().map(|(id, _)| *id));
    }
}
