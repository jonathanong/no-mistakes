use super::*;
use crate::codebase::postgres::query_annotation::evaluate::references::Edges;

pub(super) fn collect<const WARM: bool>(
    scopes: &[Scope],
    roots: RootSeeds<'_>,
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
) -> Reachable {
    let _timing = crate::codebase::postgres::query_annotation::progress::timing_and_count(
        "postgres.annotation.reachability_walk",
        "postgres.annotation.reachability_walks",
    );
    let mut found = Reachable::default();
    found.pending_env.extend(roots.environments.iter().copied());
    found.pending_args.extend(roots.arguments.iter().copied());
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
                if let Some(origins) = captured.get(&env) {
                    extend_capture_edges!(found, origins);
                }
                let cached = WARM.then(|| scopes[env].cached_references()).flatten();
                if let Some(edges) = cached {
                    extend(&mut found, edges);
                } else {
                    for value in scopes[env].values() {
                        record_raw_binding!();
                        found.value(value);
                    }
                }
                if let Some(bindings) = mapped.get(&env) {
                    found
                        .pending_args
                        .extend(bindings.iter().map(|(id, _)| *id));
                }
            }
        }
        while let Some(id) = found.pending_args.pop() {
            if found.arguments.insert(id) {
                found.identities.insert(id);
                let slots = &arena.objects[&id];
                let cached = WARM.then(|| slots.cached_references()).flatten();
                if let Some(edges) = cached {
                    extend(&mut found, edges);
                } else {
                    for value in slots.iter() {
                        record_raw_slot!();
                        found.value(value);
                    }
                }
                // Sparse slots are mutable independently of the dense snapshot.
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

fn extend(found: &mut Reachable, edges: &Edges) {
    found.pending_env.extend(edges.environments.iter().copied());
    found.pending_args.extend(edges.arguments.iter().copied());
    found.identities.extend(edges.identities.iter().copied());
}
