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

#[test]
fn foreign_receiver_projection_does_not_retain_reentrant_demands() {
    let mut collector = ImportCollector {
        foreign_receiver_writes: Some(Box::new(ForeignReceiverWrites {
            finishing: true,
            ..Default::default()
        })),
        ..Default::default()
    };
    let owner = Some(CallableId(1));
    let mut metadata = collector.receiver_mutation_metadata(1);
    metadata.context = owner;
    // Projection can revisit a mutable root; its finalization must stay finite.
    collector.retain_foreign_receiver_write(
        &(0, "box".into()),
        &[Some("client".into())],
        (1, owner),
        MutationInfo {
            metadata,
            assigned_member: Some("get"),
            completed_at: 2,
            has_assignment: true,
            proof: None,
        },
    );
    assert!(collector.foreign_receiver_writes.unwrap().writes.is_empty());
}

#[test]
fn foreign_writers_do_not_rescan_unrelated_callable_aliases() {
    for writers in [16, 64] {
        let (facts, work) = collect(&format!("unrelated-aliases-{writers}"), true);
        assert_eq!(work, [writers, writers, 256, 1]);
        let facts = facts.value_flow;
        assert_eq!(facts.reference_callable_alias_index_builds, 1);
        assert!(facts.reference_callable_alias_index_entries >= writers);
        assert!(
            facts.reference_callable_alias_visits <= 2 * writers,
            "{writers}: {} alias visits",
            facts.reference_callable_alias_visits
        );
        assert!(
            facts.receiver_mutations.as_ref().unwrap()[&0]["replica"].assignment_invalidated(
                &["get"],
                u32::MAX,
                None,
                false
            )
        );
    }
}
