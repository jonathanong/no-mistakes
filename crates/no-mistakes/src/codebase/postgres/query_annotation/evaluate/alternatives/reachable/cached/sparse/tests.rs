use super::*;
use std::cell::Cell;

thread_local! {
    static LEGACY: Cell<bool> = const { Cell::new(false) };
    static PROBES: Cell<usize> = const { Cell::new(0) };
}

pub(super) fn probe() {
    PROBES.with(|count| count.set(count.get() + 1));
}

fn measured<T>(run: impl FnOnce() -> T) -> (T, usize) {
    PROBES.with(|count| count.set(0));
    let result = run();
    (result, PROBES.with(Cell::get))
}

// Frozen pre-change prefix traversal: count actual hash lookups, including misses.
pub(super) fn legacy_prefix(
    base: usize,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
) -> Reachable {
    let mut found = Reachable::default();
    for env in 0..base {
        probe();
        if let Some(origins) = captured.get(&env) {
            found.pending_env.extend(origins.values().copied());
        }
        probe();
        if let Some(bindings) = mapped.get(&env) {
            found
                .pending_args
                .extend(bindings.iter().map(|(id, _)| *id));
        }
    }
    found
}

fn compare(
    base: usize,
    mapped: &FxHashMap<Environment, Vec<(u64, Vec<String>)>>,
    captured: &CapturedBindings,
) -> usize {
    let (old, old_probes) = measured(|| legacy_prefix(base, mapped, captured));
    let (new, new_probes) = measured(|| {
        let mut found = Reachable::default();
        auxiliary_prefix(&mut found, base, mapped, captured);
        found
    });
    assert_eq!(old_probes, 2 * base);
    assert_eq!(
        FxHashSet::from_iter(old.pending_env),
        FxHashSet::from_iter(new.pending_env)
    );
    assert_eq!(
        FxHashSet::from_iter(old.pending_args),
        FxHashSet::from_iter(new.pending_args)
    );
    assert!(new_probes <= base.min(mapped.capacity()) + base.min(captured.capacity()));
    new_probes
}

#[test]
fn empty_and_sparse_tables_do_not_probe_every_prefix_environment() {
    let base = 128;
    assert_eq!(compare(base, &Default::default(), &Default::default()), 0);
    let mapped = FxHashMap::from_iter([(0, vec![(7, vec![])]), (base + 10, vec![(99, vec![])])]);
    let captured = FxHashMap::from_iter([
        (1, FxHashMap::from_iter([("root".into(), 8)]).into()),
        (
            base + 11,
            FxHashMap::from_iter([("outside".into(), 99)]).into(),
        ),
    ]);
    assert_eq!(compare(base, &mapped, &captured), 4);
}

#[test]
fn retained_capacity_uses_bounded_prefix_lookups_for_each_table() {
    let base = 16;
    let mut mapped = FxHashMap::default();
    mapped.reserve(512);
    mapped.insert(0, vec![(7, vec![])]);
    let mut captured = CapturedBindings::default();
    captured.reserve(512);
    captured.insert(1, FxHashMap::from_iter([("root".into(), 8)]).into());
    assert_eq!(compare(base, &mapped, &captured), 2 * base);
    captured.clear();
    // Empty tables need no traversal even when clear retains oversized storage.
    assert_eq!(compare(base, &mapped, &captured), base);
    mapped.clear();
    assert_eq!(compare(base, &mapped, &captured), 0);
}

#[test]
fn zero_prefix_and_dense_tables_match_legacy_selection() {
    let mapped = FxHashMap::from_iter((0..32).map(|env| (env, vec![(env as u64, vec![])])));
    let captured = FxHashMap::from_iter(
        (0..32).map(|env| (env, FxHashMap::from_iter([("origin".into(), env)]).into())),
    );
    assert_eq!(compare(0, &mapped, &captured), 0);
    assert_eq!(compare(16, &mapped, &captured), 32);
}

#[path = "tests/liveness.rs"]
mod liveness;

pub(super) fn legacy_enabled() -> bool {
    LEGACY.with(Cell::get)
}

struct Guard(bool);
impl Guard {
    fn new(legacy: bool) -> Self {
        PROBES.with(|count| count.set(0));
        Self(LEGACY.with(|mode| mode.replace(legacy)))
    }
    fn visits(&self) -> usize {
        PROBES.with(Cell::get)
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        LEGACY.with(|mode| mode.set(self.0));
    }
}

#[path = "tests/fixture.rs"]
mod fixture;
