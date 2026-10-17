use super::*;

#[test]
fn module_provider_facts_retain_both_targets_and_physical_writer_coordinates() {
    let path = crate::test_support::rule_fixture_root("query-reached-per-item")
        .with_file_name("provenance-fixture")
        .join("src/module-writer-rebind-before.mts");
    let source = std::fs::read_to_string(path).unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let histories = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0];
    let phase = source.find("write();").unwrap() as u32;
    for (name, expected) in [("primary", false), ("replica", true)] {
        let history = &histories[name];
        assert!(history.has_assignments, "{name}: {history:#?}");
        assert_eq!(
            history.assignment_invalidated_guarded_reads(
                &["get"],
                &|_| None,
                &|| None,
                &|guards| Arc::new(guards.active_summary(|_, provider| provider.contains(phase)))
            ),
            expected,
            "{name}: {history:#?}"
        );
        for write in history
            .assigned
            .as_ref()
            .unwrap()
            .module
            .as_ref()
            .unwrap()
            .members
            .values()
            .flatten()
        {
            assert!(write.timing.single_context().flatten().is_some());
            assert!(
                write.timing.offset < phase,
                "physical writer declaration precedes the invocation"
            );
        }
    }
}

#[test]
fn module_provider_guard_merges_preserve_member_owner_and_phase_partitions() {
    use mutation_history::ValueAssignedWriter;
    let provider = |start, end| {
        Arc::new(ModuleProviderTarget {
            intervals: vec![(start, end)],
            uncertain: false,
        })
    };
    let write = |offset, owner| ValueAssignedMutation {
        offset,
        writer: ValueAssignedWriter::Callable(CallableId(owner)),
        unconditional_offset: None,
    };
    let mut history = ValueMutationHistory::default();
    history.record_module_assignment(
        &[],
        Some("get".into()),
        write(5, 1),
        provider(10, 20),
        false,
    );
    let mut later = ValueMutationHistory::default();
    later.record_module_assignment(&[], Some("get".into()), write(6, 2), provider(20, 30), true);
    later.record_module_assignment(
        &[],
        Some("info".into()),
        write(7, 2),
        provider(20, 30),
        false,
    );
    history.merge(later);
    let module = history.assigned.as_ref().unwrap().module.as_ref().unwrap();
    assert_eq!(module.members[&Some("get".into())].len(), 2);
    assert_eq!(module.members[&Some("info".into())].len(), 1);
    let first = module.active_summary(|_, target| target.contains(15));
    assert!(first.invalidated("get", &|_| None, &|| None));
    assert!(!first.invalidated("info", &|_| None, &|| None));
    let second = module.active_summary(|_, target| target.contains(25));
    assert!(second.invalidated("get", &|_| None, &|| Some(CallableId(2))));
    assert!(!second.invalidated("get", &|_| None, &|| Some(CallableId(1))));
    assert!(second.invalidated("info", &|_| None, &|| None));
    assert!(!module
        .active_summary(|_, target| target.contains(35))
        .invalidated("get", &|_| None, &|| None));
    let mut owners = fx_set();
    history.module_writer_contexts(&mut owners);
    assert_eq!(owners, [CallableId(1), CallableId(2)].into_iter().collect());
    assert!(history.assignment_invalidated(&["get"], u32::MAX, None, false));
}

#[test]
fn module_provider_dynamic_paths_keep_guarded_writes_out_of_inactive_reads() {
    use mutation_history::ValueAssignedWriter;
    let provider = Arc::new(ModuleProviderTarget {
        intervals: vec![(10, 20)],
        uncertain: false,
    });
    let mut history = ValueMutationHistory::default();
    history.record_module_assignment(
        &[Some("outer".into()), None],
        None,
        ValueAssignedMutation {
            offset: 5,
            writer: ValueAssignedWriter::Callable(CallableId(1)),
            unconditional_offset: None,
        },
        Arc::clone(&provider),
        false,
    );
    let query = |members: &[&str], phase| {
        history.assignment_invalidated_guarded_reads(members, &|_| None, &|| None, &|module| {
            Arc::new(module.active_summary(|_, target| target.contains(phase)))
        })
    };
    assert!(query(&["get", "client", "outer"], 15));
    assert!(query(&["info", "other", "outer"], 15));
    assert!(!query(&["get", "client", "outer"], 25));
    assert!(!query(&["get", "client", "unrelated"], 15));
    assert!(!query(&[], 15));
    let mut owners = fx_set();
    history.module_writer_contexts(&mut owners);
    assert_eq!(owners, [CallableId(1)].into_iter().collect());
}
