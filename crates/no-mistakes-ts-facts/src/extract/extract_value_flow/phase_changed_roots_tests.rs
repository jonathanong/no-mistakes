use super::*;
use std::cell::{Cell, RefCell};
thread_local! {
    pub(super) static LEGACY: Cell<bool> = const { Cell::new(false) };
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static COUNTS: RefCell<[usize; 10]> = const { RefCell::new([0; 10]) };
}
pub(super) fn record(index: usize) {
    if ACTIVE.with(Cell::get) {
        COUNTS.with(|c| c.borrow_mut()[index] += 1);
    }
}
pub(super) fn finish_walk(walk: &phase_walk::PhaseWalk<'_>) {
    if let Some(changed) = &walk.changed_roots {
        finish(changed.len());
    }
}
pub(super) fn finish(len: usize) {
    record(5 + len.min(2));
}
pub(super) struct LegacyRoots(FxHashSet<String>);
impl Default for LegacyRoots {
    fn default() -> Self {
        record(0);
        Self(Default::default())
    }
}
impl LegacyRoots {
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }
    pub(super) fn contains(&self, name: &str) -> bool {
        record(1);
        {
            let found = self.0.contains(name);
            if found {
                record(9);
            }
            found
        }
    }
    pub(super) fn insert(&mut self, name: String) {
        record(2);
        let capacity = self.0.capacity();
        self.0.insert(name);
        if self.0.capacity() > capacity {
            record(4);
        }
    }
}
struct Mode {
    old: bool,
}
impl Drop for Mode {
    fn drop(&mut self) {
        LEGACY.with(|v| v.set(self.old));
        ACTIVE.with(|v| v.set(false));
    }
}
fn collect(name: &str, demand: bool, legacy: bool) -> ImportFacts {
    let _mode = Mode {
        old: LEGACY.with(|v| v.replace(legacy)),
    };
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    let path = root.join("src/nested-receiver-containers").join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, &source, SourceType::from_path(&path).unwrap()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "{name}: {:?}",
        parsed.diagnostics
    );
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        demand,
    )
}
#[test]
fn changed_roots_singleton_work_guard() {
    eprintln!(
        "ACTUAL_LAYOUT legacy_field={} phase_walk={}",
        size_of::<Option<FxHashSet<String>>>(),
        size_of::<phase_walk::PhaseWalk<'_>>()
    );
    let mut budgets = Vec::new();
    for fixture in [
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
    ] {
        COUNTS.with(|v| *v.borrow_mut() = [0; 10]);
        ACTIVE.with(|v| v.set(true));
        let facts = collect(fixture, true, false);
        let counts = COUNTS.with(|v| *v.borrow());
        eprintln!("ACTUAL_WORK {fixture} contains/insert/reserved/cardinality counters={counts:?}");
        assert!(facts.value_flow.receiver_mutations.is_some());
        assert!(counts[0] > 0, "actual phase walks required: {counts:?}");
        assert_eq!(counts[0], counts[5] + counts[6] + counts[7]);
        if fixture == "mutable-pattern-phase-scale.mts" {
            assert!(
                counts[2] > 0 && counts[6] > 0,
                "phase singleton work required: {counts:?}"
            );
        }
        // Eager/cold controls may have real walks with zero changed roots.
        // Record every fixture before the final allocation-budget assertion.
        budgets.push((fixture, counts[4]));
    }
    assert!(
        budgets.iter().all(|(_, count)| *count == 0),
        "singleton hash bucket allocations: {budgets:?}"
    );
}
#[test]
fn changed_roots_all_facts_oracle() {
    for fixture in [
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-eager-class-scale.mts",
        "mutable-pattern-cold-class-scale.mts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-pattern-future-foreign-32.mts",
        "mutable-pattern-future-foreign-128.mts",
        "mutable-let-conditional-root.mts",
        "mutable-let-foreign-root.mts",
        "mutable-let-cycle.mts",
        "mutable-let-pattern-conditional-rebind.mts",
        "mutable-let-pattern-conditional-whole.mts",
        "mutable-let-pattern-class-context.mts",
        "mutable-let-pattern-shadowed-class.mts",
        "phase-changed-roots-membership.mts",
    ] {
        for demand in [false, true] {
            let old = collect(fixture, demand, true);
            COUNTS.with(|v| *v.borrow_mut() = [0; 10]);
            if fixture == "phase-changed-roots-membership.mts" && demand {
                ACTIVE.with(|v| v.set(true));
            }
            let new = collect(fixture, demand, false);
            if fixture == "phase-changed-roots-membership.mts" && demand {
                let counts = COUNTS.with(|v| *v.borrow());
                eprintln!("ACTUAL_MANY_RHS_LHS {counts:?}");
                assert!(
                    counts[7] > 0 && counts[3] > 0 && counts[8] > 0,
                    "many roots and actual read/hit required: {counts:?}"
                );
            }
            assert_eq!(old, new, "full ImportFacts {fixture}, demand={demand}");
        }
    }
}
