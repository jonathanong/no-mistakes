use super::super::{EdgeKind, VitestSetupField};

#[test]
fn public_sort_key_delegates_all_groups_and_non_vitest_detail_is_none() {
    assert_eq!(EdgeKind::Import.detail(), None);
    assert_eq!(EdgeKind::Import.sort_key(), (0, 0));
    assert_eq!(EdgeKind::HttpCall.sort_key(), (15, 0));
    assert_eq!(EdgeKind::SwiftImport.sort_key(), (21, 0));
    assert_eq!(EdgeKind::WorkflowJob.sort_key(), (30, 0));
    assert_eq!(EdgeKind::RequireResolve.sort_key(), (37, 0));
    assert_eq!(EdgeKind::WorkspaceTypeImport.sort_key(), (38, 0));
    assert_eq!(
        EdgeKind::VitestSetup(VitestSetupField::GlobalSetup).sort_key(),
        (36, 1)
    );
    assert_eq!(EdgeKind::PythonImport.sort_key(), (39, 0));
    assert_eq!(EdgeKind::PythonReference.sort_key(), (40, 0));
    assert_eq!(EdgeKind::GoImport.sort_key(), (41, 0));
    assert_eq!(EdgeKind::GoReference.sort_key(), (42, 0));
    assert_eq!(EdgeKind::RustUse.sort_key(), (43, 0));
    assert_eq!(EdgeKind::RustMod.sort_key(), (44, 0));
    assert_eq!(EdgeKind::RustPackage.sort_key(), (45, 0));
    assert_eq!(EdgeKind::RubyRequire.sort_key(), (46, 0));
    assert_eq!(EdgeKind::RubyReference.sort_key(), (47, 0));
    assert_eq!(EdgeKind::PhpUse.sort_key(), (48, 0));
    assert_eq!(EdgeKind::PhpPackage.sort_key(), (49, 0));
    assert_eq!(EdgeKind::TrpcCall.sort_key(), (50, 0));
    assert_eq!(EdgeKind::TrpcProcedure.sort_key(), (51, 0));
    assert_eq!(EdgeKind::JavaImport.sort_key(), (52, 0));
    assert_eq!(EdgeKind::JavaReference.sort_key(), (53, 0));
    assert_eq!(EdgeKind::KotlinImport.sort_key(), (54, 0));
    assert_eq!(EdgeKind::KotlinReference.sort_key(), (55, 0));
    assert_eq!(EdgeKind::ElixirImport.sort_key(), (56, 0));
    assert_eq!(EdgeKind::ElixirReference.sort_key(), (57, 0));
    assert_eq!(EdgeKind::DartImport.sort_key(), (58, 0));
    assert_eq!(EdgeKind::DartReference.sort_key(), (59, 0));
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
