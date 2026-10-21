use super::*;

fn source(name: &str) -> String {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/value-flow-guarded-array-replay/fixture/src");
    std::fs::read_to_string(root.join(format!("{name}.mts"))).unwrap()
}

#[test]
fn guarded_array_replay_work_is_output_bounded() {
    let _capture = metrics::tests::Capture::start();
    for name in ["same-width", "varying-width"] {
        let source = source(name);
        let allocator = Allocator::default();
        let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        for enabled in [false, true] {
            metrics::WORK.with(|work| work.set([0; 5]));
            let facts = extract_import_facts_from_program_with_source_and_demands(
                &parsed.program,
                &source,
                false,
                false,
                enabled,
            );
            let [events, calls, pushes, emitted, baseline] =
                metrics::WORK.with(std::cell::Cell::get);
            if !enabled {
                assert_eq!([events, calls, pushes, emitted, baseline], [0; 5]);
                continue;
            }
            let candidates = metrics::CANDIDATES.with(|values| values.borrow().clone());
            let mut distinct = Vec::new();
            for candidate in &candidates {
                if !distinct.contains(&candidate) {
                    distinct.push(candidate);
                }
            }
            // Duplicate output cannot inflate the allowed replay work.
            assert_eq!(emitted, distinct.len(), "{name}: duplicate candidates");
            let call_bound = 8 * facts.value_flow.calls.len();
            assert!(
                calls <= call_bound,
                "{name}: call scans {calls} exceed {call_bound}"
            );
            let push_bound = 8 * (distinct.len() + baseline + facts.value_flow.calls.len());
            assert!(
                pushes <= push_bound,
                "{name}: push projections {pushes} exceed {push_bound}"
            );
            let possible = facts
                .value_flow
                .possible_receiver_mutations
                .as_ref()
                .unwrap();
            for origin in ["replica", "tail"] {
                assert!(
                    possible[&0][origin].assignment_invalidated(&["get"], u32::MAX, None, false,),
                    "{name}: missing {origin}"
                );
            }
        }
    }
}

#[test]
fn guarded_array_interleaving_keeps_sparse_and_either_slots_exact() {
    let _capture = metrics::tests::Capture::start();
    let source = source("interleaved");
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let candidates = metrics::CANDIDATES.with(|values| values.borrow().clone());
    let expected = legacy::projection(&parsed.program, &source);
    assert_eq!(candidates.len(), expected.len(), "{candidates:#?}");
    assert!(
        expected.iter().all(|item| candidates.contains(item)),
        "actual {candidates:#?}; legacy {expected:#?}"
    );
    let mut holes = Vec::new();
    let mut unknown = Vec::new();
    let mut slots = candidates
        .iter()
        .filter_map(|(slot, value)| match value {
            ValueExpression::Named { name, .. } => Some((*slot, name.as_str())),
            ValueExpression::Undefined => {
                holes.push(*slot);
                None
            }
            ValueExpression::Unknown => {
                unknown.push(*slot);
                None
            }
            other => panic!("unexpected {other:?}: {candidates:#?}"),
        })
        .collect::<Vec<_>>();
    // The original expression facts retain sparse elisions as Undefined values.
    holes.sort_unstable();
    assert_eq!(holes, [0, 1]);
    unknown.sort_unstable();
    assert_eq!(unknown, [2, 3]);
    assert_eq!(candidates.len(), 9);
    slots.sort_unstable();
    assert_eq!(
        slots,
        [
            (0, "box"),
            (1, "late"),
            (2, "other"),
            (3, "late"),
            (4, "late")
        ]
    );
}

#[test]
fn guarded_array_repeated_spreads_process_each_event_once() {
    let source = source("repeated-spreads");
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    metrics::WORK.with(|work| work.set([0; 5]));
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let [events, calls, _, emitted, _] = metrics::WORK.with(std::cell::Cell::get);
    // There are 1,203 authored arrays, including the initial and final tail pair.
    assert!(events <= 2 * 1203, "replayed event prefixes: {events}");
    assert!(
        calls <= 8 * facts.value_flow.calls.len(),
        "repeated call scans: {calls}"
    );
    assert!(emitted <= 2 * 1203, "re-emitted origin prefixes: {emitted}");
}

#[path = "escaped_array_legacy_tests.rs"]
mod legacy;
