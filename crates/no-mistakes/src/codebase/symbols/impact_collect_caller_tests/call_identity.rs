use super::*;
use crate::codebase::dependencies::extract::{
    CallTargetIdentity, FunctionCall, ImportedBinding, ImportedBindingKind, InvocationKind,
};
use crate::codebase::ts_source::facts::TsFileFacts;

fn local_call(identity: CallTargetIdentity, callee: &str) -> FunctionCall {
    FunctionCall {
        caller: Some("consumer".to_string()),
        syntactic_caller: Some("consumer".to_string()),
        callee: callee.to_string(),
        line: 1,
        offset: 0,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: identity,
        static_arg: None,
        static_cwd: None,
    }
}

#[test]
fn legacy_local_caller_matching_uses_binding_identity_not_only_the_callee_spelling() {
    let local_names = BTreeSet::from(["alias".to_string()]);
    let imported = TsFileFacts {
        imported_bindings: vec![ImportedBinding {
            specifier: "./source.mts".to_string(),
            local: "alias".to_string(),
            imported: "target".to_string(),
            kind: ImportedBindingKind::Named,
            is_type_only: false,
        }],
        ..TsFileFacts::default()
    };
    let unimported = TsFileFacts::default();

    assert!(legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::ModuleExport, "alias.member"),
        &local_names,
        &imported,
    ));
    assert!(!legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::Unknown, "alias"),
        &local_names,
        &imported,
    ));
    assert!(legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::RepositoryFunction, "alias"),
        &local_names,
        &unimported,
    ));
    assert!(!legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::Global, "alias"),
        &local_names,
        &unimported,
    ));
    assert!(!legacy_call_matches_local_target(
        &local_call(CallTargetIdentity::ModuleExport, "other"),
        &local_names,
        &imported,
    ));
}
