use super::*;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/import-facts/fixture")
        .join(name)
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn ordinary_demands_skip_receiver_only_setup_sites() {
    use extract_value_flow::ordinary_demand_metrics::SITES;
    SITES.with(|counts| counts.set([0; 6]));
    for name in [
        "ordinary-demand-boundaries.mts",
        "ordinary-demand-annex.cjs",
    ] {
        let file = fixture(name);
        let source = std::fs::read_to_string(&file).unwrap();
        let allocator = Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &source, SourceType::from_path(&file).unwrap())
                .parse();
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
            false,
        );
        assert!(!facts.imports.is_empty());
        assert!(!facts.function_calls.is_empty());
        assert!(facts.value_flow.receiver_mutations.is_none());
    }
    // Actual finisher entry, conditional/default/callable/deferred state setup,
    // and assignment receiver preparation must all stay outside ordinary mode.
    assert_eq!(SITES.with(|counts| counts.get()), [0; 6]);
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn receiver_demand_still_finishes_prepared_destructuring_phases() {
    use extract_value_flow::ordinary_demand_metrics::SITES;
    let file = fixture("destructuring-phase-edges.mts");
    let source = std::fs::read_to_string(&file).unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    SITES.with(|counts| counts.set([0; 6]));
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    assert!(facts.value_flow.early_assignment_catalogs > 0);
    assert!(SITES.with(|counts| counts.get())[0] > 0);
}
