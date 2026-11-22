use crate::codebase::dependencies::extract::{
    ImportedBinding, ImportedBindingKind, ValueBinding, ValueExpression,
};
use crate::codebase::postgres::embedded::TrustedSqlTag;
use crate::codebase::ts_source::facts::TsFileFacts;

pub(super) fn is_fragment_binding(
    ts: &TsFileFacts,
    tags: &[TrustedSqlTag],
    binding: &ValueBinding,
) -> bool {
    if binding.owner.is_some() || !binding.immutable {
        return false;
    }
    let ValueExpression::Call(start, end) = binding.value else {
        return false;
    };
    trusted_tag_call(ts, tags, start, end)
}

fn trusted_tag_call(ts: &TsFileFacts, tags: &[TrustedSqlTag], start: u32, end: u32) -> bool {
    if !ts.value_flow.tagged_templates.contains_key(&(start, end)) {
        return false;
    }
    let Some(call) = ts
        .value_flow
        .calls
        .iter()
        .find(|call| call.offset == start && call.end == end)
    else {
        return false;
    };
    if call.arguments.len() != 1 {
        return false;
    }
    let ValueExpression::Named { name, .. } = &call.callee else {
        return false;
    };
    tags.iter().any(|tag| tag_matches(ts, name, tag))
}

fn tag_matches(ts: &TsFileFacts, local: &str, tag: &TrustedSqlTag) -> bool {
    if tag.module.is_empty() || tag.name.is_empty() {
        return false;
    }
    bindings(ts).any(|binding| {
        binding.local == local && !binding.is_type_only && import_is_tag(binding, tag)
    })
}

pub(super) fn imported<'a>(ts: &'a TsFileFacts, local: &str) -> Option<&'a ImportedBinding> {
    bindings(ts).find(|binding| !binding.is_type_only && binding.local == local)
}

pub(super) fn binding_is_tag(ts: &TsFileFacts, tags: &[TrustedSqlTag], local: &str) -> bool {
    tags.iter().any(|tag| tag_matches(ts, local, tag))
}

pub(super) fn exported_name(binding: &ImportedBinding) -> String {
    if binding.kind == ImportedBindingKind::Default {
        "default".to_string()
    } else {
        binding.imported.clone()
    }
}

fn bindings(ts: &TsFileFacts) -> impl Iterator<Item = &ImportedBinding> {
    ts.imported_bindings
        .iter()
        .chain(ts.value_flow.commonjs_imported_bindings.iter())
}

fn import_is_tag(binding: &ImportedBinding, tag: &TrustedSqlTag) -> bool {
    if !module_matches(&binding.specifier, &tag.module) {
        return false;
    }
    if tag.name == "default" {
        binding.kind == ImportedBindingKind::Default || binding.imported == "default"
    } else {
        binding.kind == ImportedBindingKind::Named && binding.imported == tag.name
    }
}

pub(super) fn module_matches(specifier: &str, module: &str) -> bool {
    specifier == module
        || specifier
            .strip_prefix(module)
            .is_some_and(|rest| rest.starts_with('/'))
}
