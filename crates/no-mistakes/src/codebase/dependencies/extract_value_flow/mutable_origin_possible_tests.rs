use super::*;
use std::cell::{Cell, RefCell};
thread_local! {
    static LEGACY: Cell<bool> = const { Cell::new(false) };
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static COUNTS: RefCell<[usize; 8]> = const { RefCell::new([0; 8]) };
    static PREFIXES: RefCell<Vec<(usize, Vec<usize>)>> = const { RefCell::new(Vec::new()) };
}
pub(super) fn legacy() -> bool {
    LEGACY.with(Cell::get)
}
pub(super) fn record(index: usize, amount: usize) {
    if ACTIVE.with(Cell::get) {
        COUNTS.with(|v| v.borrow_mut()[index] += amount);
    }
}
pub(super) fn prefix(limit: usize, ids: impl Iterator<Item = usize>) {
    if ACTIVE.with(Cell::get) {
        PREFIXES.with(|v| v.borrow_mut().push((limit, ids.collect())));
    }
}
struct Mode(bool);
impl Drop for Mode {
    fn drop(&mut self) {
        LEGACY.with(|v| v.set(self.0));
        ACTIVE.with(|v| v.set(false));
    }
}
fn collect(
    name: &str,
    demand: bool,
    old: bool,
) -> (ImportFacts, [usize; 8], Vec<(usize, Vec<usize>)>) {
    let _mode = Mode(LEGACY.with(|v| v.replace(old)));
    COUNTS.with(|v| *v.borrow_mut() = [0; 8]);
    PREFIXES.with(|v| v.borrow_mut().clear());
    ACTIVE.with(|v| v.set(true));
    let root = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture");
    let path = root.join("src/nested-receiver-containers").join(name);
    let source = std::fs::read_to_string(&path).unwrap();
    let allocator = Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, &source, SourceType::from_path(&path).unwrap()).parse();
    assert!(
        parsed.diagnostics.is_empty(),
        "{name}: {:?}",
        parsed.diagnostics
    );
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        demand,
    );
    let counts = COUNTS.with(|v| *v.borrow());
    let prefixes = PREFIXES.with(|v| std::mem::take(&mut *v.borrow_mut()));
    (facts, counts, prefixes)
}
#[test]
fn root_possible_hot_work_guard() {
    let (facts, counts, _) = collect("mutable-root-scale.mts", true, false);
    eprintln!("ACTUAL_POSSIBLE_HOT {counts:?}");
    assert_eq!(facts.value_flow.receiver_only_origins, 129);
    assert_eq!(facts.value_flow.receiver_only_nodes, 129);
    assert_eq!(counts[0], 129, "all unique arena leaf pushes retained");
    // Execute actual old Vec first: its buffer-growth budget must fail, while
    // the candidate still retains every occurrence and each exact prefix.
    assert_eq!(
        counts[4], 0,
        "ROOT_POSSIBLE_BUFFER_BUDGET: hot consecutive history"
    );
    assert_eq!((counts[1], counts[2], counts[3]), (1, 128, 0));
    eprintln!(
        "ACTUAL_POSSIBLE_LAYOUT header={} root_states={}",
        size_of::<PossibleOrigins>(),
        size_of::<super::super::RootStates>()
    );
}
#[test]
fn root_possible_fullfacts_and_foreign_prefix_oracle() {
    for name in [
        "mutable-root-scale.mts",
        "mutable-root-range-sparse.mts",
        "mutable-pattern-future-foreign-32.mts",
        "mutable-pattern-future-foreign-128.mts",
        "mutable-pattern-deferred-diamond.mts",
        "mutable-let-cycle.mts",
        "mutable-let-conditional-root.mts",
        "mutable-let-foreign-root.mts",
        "mutable-let-pattern-rhs-rebind.mts",
        "mutable-let-pattern-lhs-rebind.mts",
        "mutable-pattern-cold-class-scale.mts",
        "mutable-root-target-metadata.ts",
    ] {
        for demand in [false, true] {
            let (old, _, old_prefixes) = collect(name, demand, true);
            let (new, counts, new_prefixes) = collect(name, demand, false);
            assert_eq!(
                old_prefixes, new_prefixes,
                "exact actual prefix IDs and boundary {name}, demand={demand}"
            );
            assert_eq!(old, new, "full ImportFacts {name}, demand={demand}");
            if name == "mutable-root-range-sparse.mts" && demand {
                eprintln!("ACTUAL_POSSIBLE_SPARSE {counts:?}");
                assert!(
                    counts[3] > 0 && counts[5] > 0 && counts[6] > 0 && counts[7] > 0,
                    "sparse promotion and actual foreign prefix required: {counts:?}"
                );
            }
            eprintln!("POSSIBLE_FACTS_PARITY {name} demand={demand}");
        }
    }
}
#[test]
fn root_possible_exact_prefixes_match_vec_after_every_push() {
    for ids in [
        vec![],
        vec![0],
        vec![4, 5, 6],
        vec![0, 2, 3, 9],
        vec![10, 12, 20],
        vec![usize::MAX - 2, usize::MAX - 1],
    ] {
        let mut actual = PossibleOrigins::default();
        let mut old = Vec::new();
        for id in ids {
            old.push(id);
            actual.push(id);
            for limit in [
                0,
                1,
                2,
                4,
                5,
                6,
                7,
                10,
                12,
                13,
                21,
                usize::MAX - 1,
                usize::MAX,
            ] {
                let end = old.partition_point(|id| *id < limit);
                assert_eq!(
                    actual.before(limit).collect::<Vec<_>>(),
                    old[..end],
                    "limit={limit}"
                );
            }
        }
    }
}
#[test]
fn root_possible_overflow_is_checked_before_mutation() {
    let mut actual = PossibleOrigins::default();
    actual.push(5);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| actual.push(usize::MAX)));
    assert!(result.is_err());
    assert_eq!(actual.before(usize::MAX).collect::<Vec<_>>(), vec![5]);
    assert_eq!(size_of::<PossibleOrigins>(), size_of::<Vec<usize>>());
}
