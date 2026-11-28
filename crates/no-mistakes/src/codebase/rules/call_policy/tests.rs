use super::root::{validate_root_options, FileRoot, Root};
use super::{call_argument_values, site_source_node};
use crate::codebase::dependencies::extract::{CallableId, InvocationKind};
use crate::codebase::dependencies::graph::{NodeId, ResolvedCallSite, ResolvedCallTarget};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_source::module_facts::arguments::ArgumentValues;
use crate::codebase::ts_source::module_facts::{ModuleDiagnostic, TypeScriptModuleFacts};
use crate::config::v2::schema::StringOrList;
use crate::fx::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn validate_root_options_rejects_empty_roots_and_blank_exclude_roots() {
    let empty_roots = validate_root_options(&[], None).unwrap_err().to_string();
    assert!(
        empty_roots.contains("requires at least one options.roots selector"),
        "{empty_roots}"
    );

    let file_root = Root::File(FileRoot {
        file: "src/app.ts".into(),
    });
    let empty_exclude = validate_root_options(&[file_root], Some(&StringOrList::Many(Vec::new())))
        .unwrap_err()
        .to_string();
    assert!(
        empty_exclude.contains("excludeRoots must contain non-empty glob patterns"),
        "{empty_exclude}"
    );

    let file_root = Root::File(FileRoot {
        file: "src/app.ts".into(),
    });
    let blank_exclude = validate_root_options(&[file_root], Some(&StringOrList::One("   ".into())))
        .unwrap_err()
        .to_string();
    assert!(
        blank_exclude.contains("excludeRoots must contain non-empty glob patterns"),
        "{blank_exclude}"
    );

    let file_root = Root::File(FileRoot {
        file: "src/app.ts".into(),
    });
    assert!(
        validate_root_options(&[file_root], Some(&StringOrList::One("src/gen/**".into()))).is_ok()
    );
}

#[test]
fn call_argument_values_withholds_facts_on_errors_and_unsafe_diagnostics() {
    let mut args = FxHashMap::default();
    args.insert(
        10,
        ArgumentValues {
            values: Vec::new(),
            spread_from: None,
            path: None,
        },
    );
    let mut module = TypeScriptModuleFacts {
        call_arguments: Some(Arc::new(args)),
        ..TypeScriptModuleFacts::default()
    };

    let parse_failed = TsFileFacts {
        fatal_parse_error: true,
        module_bindings: Some(Arc::new(module.clone())),
        ..TsFileFacts::default()
    };
    assert!(call_argument_values(&parse_failed, 10).is_none());

    module.diagnostics.push(ModuleDiagnostic {
        kind: "semanticError".into(),
        message: "duplicate binding".into(),
        span: None,
    });
    let unsafe_diag = TsFileFacts {
        module_bindings: Some(Arc::new(module)),
        ..TsFileFacts::default()
    };
    assert!(call_argument_values(&unsafe_diag, 10).is_none());
}

#[test]
fn site_source_node_maps_file_symbol_and_callable_callers() {
    let file = PathBuf::from("src/app.ts");
    let mut site = ResolvedCallSite {
        file: file.clone(),
        caller: None,
        caller_id: None,
        line: 1,
        offset: 0,
        invocation: InvocationKind::Call,
        source_callee: "fetch".into(),
        target: ResolvedCallTarget::Global {
            name: "fetch".into(),
        },
        alternatives: Vec::new(),
        returned_from: Vec::new(),
        typed_from: Vec::new(),
        returned_member_from: Vec::new(),
        target_node: None,
    };
    assert_eq!(site_source_node(&site), NodeId::file(&file));

    site.caller = Some("run".into());
    assert_eq!(site_source_node(&site), NodeId::symbol(&file, "run"));

    site.caller_id = Some(CallableId(1));
    assert_eq!(
        site_source_node(&site),
        NodeId::callable(&file, "run", CallableId(1))
    );
}
