use crate::codebase::check_facts::{CheckFactMap, CheckFileFacts};
use crate::codebase::dependencies::extract::{
    CallTargetIdentity, ExportedBinding, ExtractedImport, FunctionCall, ImportKind,
    ImportedBinding, ImportedBindingKind, InvocationKind, ValueBinding, ValueExpression,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use std::path::Path;
use std::sync::Arc;

pub(super) fn file_facts(ts: TsFileFacts) -> Arc<CheckFileFacts> {
    CheckFileFacts {
        ts: Arc::new(ts),
        ..Default::default()
    }
    .into()
}

pub(super) fn import(specifier: &str, kind: ImportKind, computed: bool) -> ExtractedImport {
    ExtractedImport {
        specifier: specifier.into(),
        kind,
        line: 1,
        function_scope: None,
        function_scope_id: None,
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
        computed,
    }
}

pub(super) fn named_import(local: &str, imported: &str, specifier: &str) -> ImportedBinding {
    ImportedBinding {
        specifier: specifier.into(),
        local: local.into(),
        imported: imported.into(),
        kind: ImportedBindingKind::Named,
        is_type_only: false,
    }
}

pub(super) fn require_binding(name: &str, offset: u32) -> ValueBinding {
    ValueBinding {
        immutable: true,
        owner: None,
        name: name.into(),
        scope: 0,
        declared_at: 1,
        repeated: false,
        invalidated_at: None,
        mutation_contexts: crate::fx::fx_set(),
        value: ValueExpression::Call(offset, offset + 1),
    }
}

pub(super) fn require_call(offset: u32, specifier: Option<&str>) -> FunctionCall {
    FunctionCall {
        caller: None,
        caller_id: None,
        syntactic_caller: None,
        callee: "require".into(),
        line: 1,
        offset,
        is_callback: false,
        invocation: InvocationKind::Call,
        target_identity: CallTargetIdentity::Global,
        callee_binding_scope: None,
        static_arg: specifier.map(str::to_owned),
        static_cwd: None,
    }
}

pub(super) fn export(specifier: Option<&str>, local: &str, exported: &str) -> ExportedBinding {
    ExportedBinding {
        specifier: specifier.map(str::to_owned),
        local: local.into(),
        exported: exported.into(),
    }
}

pub(super) fn insert(facts: &mut CheckFactMap, path: &Path, ts: TsFileFacts) {
    facts.files.push(path.to_path_buf());
    facts.ts.insert(path.to_path_buf(), file_facts(ts));
}
