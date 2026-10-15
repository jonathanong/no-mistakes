use super::*;

fn write(offset: u32, owner: Option<CallableId>) -> ValueAssignedMutation {
    ValueAssignedMutation {
        offset,
        unconditional_offset: std::num::NonZeroU32::new(offset),
        writer: ValueAssignedWriter::single(owner),
    }
}
#[test]
fn singleton_promotes_and_merges_without_losing_context_projections() {
    let mut target = ValuePropagatedMutations::default();
    target.merge(ValuePropagatedMutations::default());
    assert!(target.context(None).is_none());
    let mut first = ValuePropagatedMutations::default();
    first.record(Some("get".into()), write(20, None));
    target.merge(first.clone());
    target.merge(first);
    assert!(target.contexts.is_empty());
    assert_eq!(target.context(None).unwrap(), target.aggregate());
    target.record(None, write(30, Some(CallableId(1))));
    assert!(target.context(None).unwrap().dynamic.is_none());
    assert!(
        target
            .context(Some(CallableId(1)))
            .unwrap()
            .dynamic
            .is_some()
    );
    assert!(target.context(Some(CallableId(2))).is_none());
    let mut other = ValuePropagatedMutations::default();
    other.record(Some("get".into()), write(10, Some(CallableId(2))));
    target.merge(other);
    assert_eq!(target.aggregate().members["get"].offset, 10);
    assert_eq!(
        target.aggregate().members["get"].writer,
        ValueAssignedWriter::Mixed
    );
    assert_eq!(target.context(None).unwrap().members["get"].offset, 20);
    let mut empty = ValuePropagatedMutations::default();
    empty.merge(target.clone());
    empty.merge(target.clone());
    assert_eq!(empty.aggregate().members, target.aggregate().members);
    assert_eq!(empty.aggregate().dynamic, target.aggregate().dynamic);
    for owner in [None, Some(CallableId(1)), Some(CallableId(2))] {
        assert_eq!(
            empty.context(owner).unwrap().members,
            target.context(owner).unwrap().members
        );
        assert_eq!(
            empty.context(owner).unwrap().dynamic,
            target.context(owner).unwrap().dynamic
        );
    }
}
#[test]
#[cfg(feature = "test-instrumentation")]
fn physical_mutable_root_fixture_records_singleton_once_per_write() {
    let source = std::fs::read_to_string(
        crate::test_support::rule_fixture_root("query-reached-per-item")
            .with_file_name("provenance-fixture")
            .join("src/nested-receiver-containers/mutable-root-scale.mts"),
    )
    .unwrap();
    let allocator = Allocator::default();
    let parsed = oxc_parser::Parser::new(&allocator, &source, SourceType::ts()).parse();
    assert!(parsed.diagnostics.is_empty());
    let facts = extract_import_facts_from_program_with_source_and_demands(
        &parsed.program,
        &source,
        false,
        false,
        true,
    );
    let history = &facts.value_flow.receiver_mutations.as_ref().unwrap()[&0]["client"];
    let propagated = history
        .assigned
        .as_ref()
        .unwrap()
        .propagated
        .as_ref()
        .unwrap();
    assert_eq!(propagated.records, 128);
    assert!(propagated.contexts.is_empty());
    let retained_member_maps = usize::from(propagated.aggregate.members.capacity() > 0)
        + usize::from(propagated.contexts.capacity() > 0)
        + propagated
            .contexts
            .values()
            .filter(|history| history.members.capacity() > 0)
            .count();
    assert_eq!(retained_member_maps, 1);
    assert!(history.assignment_invalidated(&["get"], source.len() as u32, None, false));
}
#[test]
fn debug_preserves_logical_aggregate_and_context_shape() {
    struct Legacy<'a> {
        aggregate: &'a ValueAssignedMutations,
        contexts: FxHashMap<Option<CallableId>, &'a ValueAssignedMutations>,
    }
    impl std::fmt::Debug for Legacy<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ValuePropagatedMutations")
                .field("aggregate", self.aggregate)
                .field("contexts", &self.contexts)
                .finish()
        }
    }
    let mut writes = ValuePropagatedMutations::default();
    for owner in [None, Some(None), Some(Some(CallableId(1)))] {
        if let Some(owner) = owner {
            writes.record(None, write(10, owner));
        }
        let mut contexts = fx_map();
        for key in [None, Some(CallableId(1))] {
            if let Some(history) = writes.context(key) {
                contexts.insert(key, history);
            }
        }
        let old = Legacy {
            aggregate: writes.aggregate(),
            contexts,
        };
        assert_eq!(format!("{writes:#?}"), format!("{old:#?}"));
    }
}

#[test]
fn callable_singleton_retains_owner_through_mixed_summary_merge() {
    let mut target = ValuePropagatedMutations::default();
    target.record(Some("get".into()), write(20, Some(CallableId(1))));
    target.record(None, write(30, Some(CallableId(1))));
    assert!(target.context(None).is_none());
    assert_eq!(
        target.context(Some(CallableId(1))),
        Some(target.aggregate())
    );
    let mut source = ValuePropagatedMutations::default();
    source.record(Some("get".into()), write(10, None));
    source.record(Some("set".into()), write(40, Some(CallableId(2))));
    target.merge(source);
    assert_eq!(
        target.aggregate().members["get"].writer,
        ValueAssignedWriter::Mixed
    );
    assert_eq!(
        target.context(Some(CallableId(1))).unwrap().members["get"].offset,
        20
    );
    assert_eq!(target.context(None).unwrap().members["get"].offset, 10);
    assert!(
        target
            .context(Some(CallableId(2)))
            .unwrap()
            .members
            .contains_key("set")
    );
}

#[test]
fn distinct_singletons_merge_into_separate_contexts() {
    let mut module = ValuePropagatedMutations::default();
    module.record(Some("get".into()), write(10, None));
    let mut callable = ValuePropagatedMutations::default();
    callable.record(None, write(20, Some(CallableId(1))));
    module.merge(callable);
    assert!(module.single.is_none());
    assert_eq!(module.contexts.len(), 2);
    assert!(module.context(None).unwrap().dynamic.is_none());
    assert!(
        module
            .context(Some(CallableId(1)))
            .unwrap()
            .dynamic
            .is_some()
    );
}
