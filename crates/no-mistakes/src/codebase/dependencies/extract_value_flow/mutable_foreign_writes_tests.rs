use super::*;

fn source(name: &str) -> String {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/foreign-receiver-declaration-order/fixture/src");
    std::fs::read_to_string(root.join(format!("{name}.mts"))).unwrap()
}

fn collect(name: &str, demand: bool) -> (ImportFacts, [usize; 4]) {
    WORK.with(|work| work.set([0; 4]));
    let source = source(name);
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

#[test]
fn foreign_root_demands_share_future_provider_work_and_stay_cold() {
    for providers in [16, 256] {
        let (facts, work) = collect(&format!("scale-{providers}"), true);
        assert_eq!(
            work,
            [32, 1, providers, 1],
            "physical/group/provider/projected work"
        );
        let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["replica"];
        assert!(history.assignment_invalidated(&["get"], u32::MAX, None, false));
        assert_eq!(collect(&format!("scale-{providers}"), false).1, [0; 4]);
    }
    assert_eq!(collect("cold", true).1, [0; 4]);
    assert_eq!(collect("captured-root", true).1, [0; 4]);
    assert_eq!(
        size_of::<Option<Box<ForeignReceiverWrites>>>(),
        size_of::<usize>()
    );
}

#[test]
fn distinct_writers_share_provider_projection_across_watermarks() {
    for writers in [16, 64] {
        assert_eq!(
            collect(&format!("writers-{writers}"), true).1,
            [writers, writers, 256, 1]
        );
    }
    assert_eq!(collect("interleaved", true).1, [32, 32, 32, 1]);
}
