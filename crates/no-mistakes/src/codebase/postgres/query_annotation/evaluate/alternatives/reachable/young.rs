//! A checkpoint's old graph is pinned; only remembered writes can point into youth.
use super::super::super::frame_arena::{FrameCheckpoint, ScopeArena};
use super::*;

#[derive(Clone, Copy)]
pub(in super::super) struct Frozen {
    pub frames: FrameCheckpoint,
    pub identities: u64,
}

pub(in super::super) fn collect_young(
    scopes: &ScopeArena,
    frozen: Frozen,
    roots: RootSeeds<'_>,
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> Reachable {
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment("postgres.annotation.reachability_walks", 1);
    }
    let mut found = Reachable::default();
    found.pending_env.extend(roots.environments.iter().copied());
    found.pending_args.extend(roots.arguments.iter().copied());
    for root in roots.transient {
        match root {
            TransientRoot::Argument(id) => found.pending_args.push(*id),
            TransientRoot::Environment(env) => found.pending_env.push(*env),
            TransientRoot::Builder(id) if *id >= frozen.identities => {
                found.identities.insert(*id);
            }
            TransientRoot::Builder(_) => {}
        }
    }
    for value in returned {
        found.value(value);
    }
    while !found.pending_env.is_empty() || !found.pending_args.is_empty() {
        while let Some(env) = found.pending_env.pop() {
            if scopes.is_before(env, frozen.frames) || !found.environments.insert(env) {
                continue;
            }
            let scope = scopes.get(env).expect("reachable live environment");
            if let Some(origins) = captured.get(&env) {
                found.pending_env.extend(origins.values().copied());
            }
            for value in scope.values() {
                found.value(value);
            }
            if let Some(bindings) = mapped.get(&env) {
                found
                    .pending_args
                    .extend(bindings.iter().map(|(id, _)| *id));
            }
        }
        while let Some(id) = found.pending_args.pop() {
            if id < frozen.identities || !found.arguments.insert(id) {
                continue;
            }
            found.identities.insert(id);
            for value in arena.dense(id).iter().chain(
                arena
                    .extras
                    .get(&id)
                    .into_iter()
                    .flat_map(|slots| slots.values()),
            ) {
                found.value(value);
            }
        }
    }
    found.identities.retain(|id| *id >= frozen.identities);
    if let Some(observer) = observer {
        observer.increment(
            "postgres.annotation.young_frame_visits",
            found.environments.len() as u64,
        );
        observer.increment(
            "postgres.annotation.young_argument_visits",
            found.arguments.len() as u64,
        );
    }
    found
}
