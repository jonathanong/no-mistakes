use super::*;

thread_local! {
    pub(super) static LIMIT_OVERRIDE: std::cell::Cell<Option<usize>> = const { std::cell::Cell::new(None) };
    static WORK: std::cell::Cell<[usize; 2]> = const { std::cell::Cell::new([0; 2]) };
}
pub(super) fn record(index: usize, amount: usize) {
    WORK.with(|work| {
        let mut values = work.get();
        values[index] += amount;
        work.set(values);
    });
}
struct Mode(Option<usize>);
impl Drop for Mode {
    fn drop(&mut self) {
        LIMIT_OVERRIDE.with(|mode| mode.set(self.0));
    }
}
fn collect(
    file: &std::path::Path,
    limit: Option<usize>,
    demand: bool,
) -> (ImportFacts, [usize; 2]) {
    let _mode = Mode(LIMIT_OVERRIDE.with(|mode| mode.replace(limit)));
    WORK.with(|work| work.set([0; 2]));
    let source = std::fs::read_to_string(file).unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        demand,
    );
    (facts, WORK.with(std::cell::Cell::get))
}
fn root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/foreign-receiver-work-budget/fixture/src")
}
#[test]
fn shared_receiver_work_stays_bounded_as_writer_count_grows() {
    for writers in [2048, 4096] {
        let file = root().join(format!("shared-{writers}.mts"));
        let (bounded, work) = collect(&file, None, true);
        assert!(bounded
            .value_flow
            .incomplete_reason
            .as_deref()
            .unwrap()
            .contains("exclude vendored or minified assets"));
        assert!(bounded.value_flow.bindings.is_empty());
        assert!(bounded.value_flow.receiver_mutations.is_none());
        assert!(work[0] <= LIMIT);
        assert_eq!(work[1], 1);
        let (complete, unbounded) = collect(&file, Some(usize::MAX), true);
        assert!(complete.value_flow.incomplete_reason.is_none());
        assert!(unbounded[0] > LIMIT, "{writers}: {unbounded:?}");
        println!("FOREIGN_BUDGET writers={writers} bounded={work:?} unlimited={unbounded:?}");
        assert_eq!(collect(&file, None, false).1, [0; 2]);
    }
}
#[test]
fn ordinary_files_keep_identical_complete_facts() {
    let controls = root().with_file_name("src");
    for name in ["ordinary.mts", "shared-16.mts", "shared-64.mts"] {
        let file = controls.join(name);
        for demand in [false, true] {
            let (bounded, _) = collect(&file, None, demand);
            let (unlimited, _) = collect(&file, Some(usize::MAX), demand);
            assert!(bounded.value_flow.incomplete_reason.is_none());
            assert_eq!(bounded, unlimited, "{name} demand={demand}");
        }
    }
    let controls = root().join("../../../foreign-receiver-declaration-order/fixture/src");
    for file in std::fs::read_dir(controls).unwrap() {
        let file = file.unwrap().path();
        if file.extension().is_some_and(|extension| extension == "mts") {
            assert_eq!(
                collect(&file, None, true).0,
                collect(&file, Some(usize::MAX), true).0,
                "{}",
                file.display()
            );
        }
    }
}
#[test]
fn exhausted_outputs_and_zero_remaining_never_extend_partial_facts() {
    let _mode = Mode(LIMIT_OVERRIDE.with(|mode| mode.replace(Some(1))));
    let mut budget = WorkBudget::default();
    assert_eq!(budget.charge(1), Some(()));
    assert_eq!(budget.charge(1), None);
    assert_eq!(budget.charge(0), None);
    let mut output = budget.output();
    output.extend([(((0, "receiver".into()), Vec::new()), Vec::new(), (0, None))]);
    assert!(output.finish().is_none());
}

#[test]
fn exhaustion_at_preflight_construction_and_replay_is_explicit() {
    let file = root().join("shared-16.mts");
    let (complete, work) = collect(&file, Some(usize::MAX), true);
    for limit in [
        0,
        1,
        16,
        32,
        64,
        128,
        192,
        256,
        512,
        1024,
        2048,
        work[0] - 1,
    ] {
        let (facts, bounded) = collect(&file, Some(limit), true);
        assert!(
            facts.value_flow.incomplete_reason.is_some(),
            "limit={limit}"
        );
        assert!(facts.value_flow.bindings.is_empty());
        assert!(bounded[0] <= limit);
        assert_eq!(bounded[1], 1);
    }
    assert_eq!(collect(&file, Some(work[0]), true).0, complete);
}

#[test]
fn parameter_provider_diamonds_stop_expansion_before_retaining_exponential_outputs() {
    for depth in [16, 20] {
        let (facts, work) = collect(
            &root().join(format!("argument-diamond-{depth}.mts")),
            None,
            true,
        );
        assert!(
            facts.value_flow.incomplete_reason.is_some(),
            "depth={depth}, work={work:?}"
        );
        assert!(work[0] <= LIMIT);
        assert_eq!(work[1], 1);
        println!("FOREIGN_DIAMOND depth={depth} bounded={work:?}");
    }
}

#[test]
fn parameter_provider_construction_exhaustion_cannot_return_partial_facts() {
    let file = root().join("argument-diamond-16.mts");
    for limit in [0, 1, 2, 3, 4, 5, 8, 16, 32, 64, 128, 256, 1024] {
        let (facts, work) = collect(&file, Some(limit), true);
        assert!(facts.value_flow.incomplete_reason.is_some());
        assert!(facts.value_flow.bindings.is_empty());
        assert!(work[0] <= limit);
        assert_eq!(work[1], 1);
    }
}
