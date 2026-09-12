use super::*;
use crate::codebase::dependencies::extract::{
    CallTargetIdentity, FunctionCall, ImportedBinding, ImportedBindingKind, InvocationKind,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_symbols::{Export, FileSymbols};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

fn call(callee: &str, identity: CallTargetIdentity) -> FunctionCall {
    FunctionCall {
        caller: None,
        caller_id: None,
        syntactic_caller: None,
        callee: callee.to_string(),
        line: 1,
        offset: 0,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: identity,
        callee_binding_scope: None,
        static_arg: None,
        static_cwd: None,
    }
}

#[test]
fn caller_helper_predicates_cover_test_files_exports_and_identities() {
    assert!(is_test_like_file(Path::new("src/foo.test.ts")));
    assert!(is_test_like_file(Path::new("src/bar.spec.mts")));
    assert!(!is_test_like_file(Path::new("src/foo.ts")));
    assert!(!is_test_like_file(Path::new("")));
    assert!(!is_test_like_file(Path::new("..")));
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        assert!(!is_test_like_file(Path::new(std::ffi::OsStr::from_bytes(
            b"foo.test.\xff.ts"
        ))));
    }

    let symbols = FileSymbols {
        exports: vec![Export {
            name: "run".to_string(),
            local: Some("impl".to_string()),
            kind: ExportKind::Const,
            line: 1,
            is_type_only: false,
        }],
        imports: vec![],
    };
    let path = PathBuf::from("/repo/src/a.ts");
    let mut target_symbols = BTreeMap::new();
    assert!(!caller_is_target_export(
        &symbols,
        &path,
        &target_symbols,
        "run"
    ));
    target_symbols.insert(
        path.clone(),
        BTreeSet::from(["run".to_string(), "impl".to_string()]),
    );
    assert!(caller_is_target_export(
        &symbols,
        &path,
        &target_symbols,
        "run"
    ));
    assert!(caller_is_target_export(
        &symbols,
        &path,
        &target_symbols,
        "impl"
    ));
    assert!(!caller_is_target_export(
        &symbols,
        &path,
        &target_symbols,
        "missing"
    ));

    let locals = BTreeSet::from(["parseDate".to_string()]);
    assert!(matches_local_callee("parseDate", &locals));
    assert!(matches_local_callee("parseDate.format", &locals));
    assert!(!matches_local_callee("parse", &locals));

    let facts = TsFileFacts {
        imported_bindings: vec![ImportedBinding {
            specifier: "./utils.mts".to_string(),
            local: "parseDate".to_string(),
            imported: "parseDate".to_string(),
            kind: ImportedBindingKind::Named,
            is_type_only: false,
        }],
        ..Default::default()
    };
    assert!(!legacy_call_matches_local_target(
        &call("parseDate", CallTargetIdentity::RepositoryFunction),
        &locals,
        &facts,
    ));
    assert!(legacy_call_matches_local_target(
        &call("parseDate", CallTargetIdentity::ModuleExport),
        &locals,
        &facts,
    ));
    assert!(!legacy_call_matches_local_target(
        &call("parseDate", CallTargetIdentity::Unknown),
        &locals,
        &facts,
    ));
    assert!(!legacy_call_matches_local_target(
        &call("parseDate", CallTargetIdentity::Global),
        &locals,
        &facts,
    ));
    assert!(!legacy_call_matches_local_target(
        &call("other", CallTargetIdentity::ModuleExport),
        &locals,
        &facts,
    ));

    let local_only = BTreeSet::from(["helper".to_string()]);
    assert!(legacy_call_matches_local_target(
        &call("helper", CallTargetIdentity::RepositoryFunction),
        &local_only,
        &TsFileFacts::default(),
    ));
    assert!(legacy_call_matches_local_target(
        &call("helper.run", CallTargetIdentity::RepositoryFunction),
        &local_only,
        &TsFileFacts::default(),
    ));
}
