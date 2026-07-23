use super::super::{EdgeKind, VitestSetupField};

#[test]
fn public_sort_key_delegates_all_groups_and_non_vitest_detail_is_none() {
    assert_eq!(EdgeKind::Import.detail(), None);
    assert_eq!(EdgeKind::Import.sort_key(), (0, 0));
    assert_eq!(EdgeKind::HttpCall.sort_key(), (15, 0));
    assert_eq!(EdgeKind::SwiftImport.sort_key(), (21, 0));
    assert_eq!(EdgeKind::WorkflowJob.sort_key(), (30, 0));
    assert_eq!(
        EdgeKind::VitestSetup(VitestSetupField::GlobalSetup).sort_key(),
        (36, 1)
    );
}

#[test]
#[should_panic(expected = "core edge group is exhaustive")]
fn core_rejects_a_kind_outside_its_group() {
    let _ = super::core(EdgeKind::HttpCall);
}

#[test]
#[should_panic(expected = "runtime edge group is exhaustive")]
fn runtime_rejects_a_kind_outside_its_group() {
    let _ = super::runtime(EdgeKind::Import);
}

#[test]
#[should_panic(expected = "language edge group is exhaustive")]
fn language_rejects_a_kind_outside_its_group() {
    let _ = super::language(EdgeKind::Import);
}

#[test]
#[should_panic(expected = "workflow edge group is exhaustive")]
fn workflow_rejects_a_kind_outside_its_group() {
    let _ = super::workflow(EdgeKind::Import);
}
