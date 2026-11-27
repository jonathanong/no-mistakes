use super::*;
use std::cell::Cell;
thread_local! {
    pub(super) static LEGACY: Cell<bool> = const { Cell::new(false) };
    static COUNTS: Cell<[usize; 4]> = const { Cell::new([0; 4]) };
}
pub(super) fn writer(admitted: bool) {
    COUNTS.with(|counts| {
        let mut values = counts.get();
        values[0] += 1;
        values[2 + usize::from(admitted)] += 1;
        counts.set(values);
    });
}
pub(in super::super) fn query() {
    COUNTS.with(|counts| {
        let mut values = counts.get();
        values[1] += 1;
        counts.set(values);
    });
}
// Only the oracle re-reads the catalog per node; production uses its const admission.
pub(super) fn admitted<const RHS: bool>(collector: &ImportCollector) -> bool {
    if LEGACY.with(Cell::get) {
        collector.projection_rhs_origins()
    } else {
        RHS
    }
}
struct Mode(bool);
impl Drop for Mode {
    fn drop(&mut self) {
        LEGACY.with(|mode| mode.set(self.0));
    }
}
fn collect(source: &str, legacy: bool, demand: bool) -> (ImportFacts, [usize; 4]) {
    let _mode = Mode(LEGACY.with(|mode| mode.replace(legacy)));
    COUNTS.with(|counts| counts.set([0; 4]));
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        source,
        false,
        false,
        demand,
    );
    (facts, COUNTS.with(Cell::get))
}
#[test]
fn one_catalog_admission_preserves_every_writer_fact() {
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture")
        .join("src/nested-receiver-containers");
    let mut none = 0;
    let mut some = 0;
    let mut budgets = Vec::new();
    for name in [
        "mutable-root-scale.mts",
        "mutable-pattern-phase-scale.mts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-projection-wide.ts",
        "assignment-rhs-direct.mts",
        "assignment-rhs-restored.mts",
        "assignment-rhs-conditional.mts",
        "assignment-rhs-cycle.mts",
        "assignment-rhs-child-restored.mts",
        "assignment-rhs-saved-choice.mts",
        "assignment-rhs-diamond-8.mts",
        "assignment-rhs-rebound-parent-scope.mts",
    ] {
        let source = std::fs::read_to_string(root.join(name)).unwrap();
        for demand in [false, true] {
            let (old, old_counts) = collect(&source, true, demand);
            let (new, counts) = collect(&source, false, demand);
            assert_eq!(old, new, "{name}: demand={demand}");
            none += counts[2];
            some += counts[3];
            println!("ADMISSION {name} demand={demand} old={old_counts:?} new={counts:?}");
            budgets.push(counts[1] <= counts[0]);
        }
    }
    assert!(none > 0 && some > 0, "both actual catalog states required");
    assert!(
        budgets.into_iter().all(|valid| valid),
        "one catalog query per writer budget"
    );
}
