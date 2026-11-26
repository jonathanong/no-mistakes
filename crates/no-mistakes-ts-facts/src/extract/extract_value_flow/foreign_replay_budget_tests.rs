use super::*;
thread_local! { static REPLAYS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
pub(super) fn count(amount: usize) {
    REPLAYS.with(|count| count.set(count.get() + amount));
}
fn reference() -> nested::ReceiverWrite {
    (((0, "receiver".into()), Vec::new()), Vec::new(), (0, None))
}
fn collector(exhausted: bool) -> ImportCollector {
    REPLAYS.with(|count| count.set(0));
    let mut work = WorkBudget::default();
    let remaining = work.remaining();
    work.charge(remaining).unwrap();
    if exhausted {
        assert!(work.charge(1).is_none());
    }
    ImportCollector {
        foreign_receiver_writes: Some(Box::new(ForeignReceiverWrites::finishing_with(work))),
        ..Default::default()
    }
}
fn index() -> ForeignProviderIndex {
    let mut index = ForeignProviderIndex::default();
    index.module_references = vec![
        (
            reference(),
            std::sync::Arc::new(ModuleProviderTarget::default()),
        ),
        (
            reference(),
            std::sync::Arc::new(ModuleProviderTarget::default()),
        ),
    ];
    index
}

fn info(collector: &mut ImportCollector) -> MutationInfo<'static> {
    MutationInfo {
        metadata: collector.receiver_mutation_metadata(1),
        completed_at: 2,
        assigned_member: None,
        has_assignment: true,
        proof: None,
    }
}
#[test]
fn exhausted_replays_do_not_clone_or_project_further_module_and_possible_references() {
    let mut collector = collector(true);
    let info = info(&mut collector);
    let mut aliases = None;
    collector.replay_foreign(
        &index(),
        vec![reference()],
        CallableId(1),
        info,
        &mut aliases,
    );
    collector.replay_possible_foreign(&index(), None, info);
    assert_eq!(REPLAYS.with(std::cell::Cell::get), 0);
    assert!(aliases.is_none());
    assert!(collector.active_module_provider.is_none());
}
#[test]
fn module_replay_exhaustion_skips_remaining_modules_and_alias_construction() {
    let mut collector = collector(false);
    let info = info(&mut collector);
    let mut aliases = None;
    collector.replay_foreign(
        &index(),
        vec![reference()],
        CallableId(1),
        info,
        &mut aliases,
    );
    assert_eq!(REPLAYS.with(std::cell::Cell::get), 1);
    assert!(collector.foreign_work_exhausted());
    assert!(aliases.is_none());
    assert!(collector.active_module_provider.is_none());
}

#[test]
fn possible_provider_scans_are_charged_before_reference_clones() {
    let mut collector = collector(false);
    let info = info(&mut collector);
    collector.replay_possible_foreign(&index(), None, info);
    assert_eq!(REPLAYS.with(std::cell::Cell::get), 0);
    assert!(collector.foreign_work_exhausted());
}
