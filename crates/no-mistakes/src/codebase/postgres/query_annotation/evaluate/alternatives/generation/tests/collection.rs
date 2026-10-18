//! Test-only full-walk oracle: the production collector never scans frozen frames.
use super::super::super::{frame_arena::ScopeArena, Environment, Value};
use super::super::{arena::Arena, reachable};
use crate::fx::FxHashMap;
use std::cell::Cell;
thread_local! {
    static FULL: Cell<bool> = const { Cell::new(false) };
}
struct Restore(bool);
impl Drop for Restore {
    fn drop(&mut self) {
        FULL.with(|mode| mode.set(self.0));
    }
}
pub(in super::super) fn with_collection_mode<T>(full: bool, run: impl FnOnce() -> T) -> T {
    let _restore = Restore(FULL.with(|mode| mode.replace(full)));
    run()
}
pub(super) fn pin_cutoff(cutoff: u64) -> u64 {
    if FULL.with(Cell::get) {
        0
    } else {
        cutoff
    }
}
pub(super) fn collect(
    scopes: &ScopeArena,
    frozen: reachable::Frozen,
    roots: reachable::RootSeeds<'_>,
    returned: &[Value],
    arena: Arena<'_>,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &FxHashMap<Environment, FxHashMap<String, Environment>>,
) -> reachable::Reachable {
    if !FULL.with(Cell::get) {
        return reachable::collect_young(scopes, frozen, roots, returned, arena, mapped, captured);
    }
    let mut environments = roots.environments.to_vec();
    environments.extend(
        scopes
            .ids()
            .filter(|env| scopes.is_before(*env, frozen.frames)),
    );
    let found = reachable::collect_from_roots(
        scopes,
        reachable::RootSeeds {
            environments: &environments,
            ..roots
        },
        returned,
        arena,
        mapped,
        captured,
    );
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(
            "postgres.annotation.generation_full_frame_visits",
            found.environments.len() as u64,
        );
        observer.increment(
            "postgres.annotation.generation_full_old_frame_visits",
            found
                .environments
                .iter()
                .filter(|env| scopes.is_before(**env, frozen.frames))
                .count() as u64,
        );
    }
    found
}
