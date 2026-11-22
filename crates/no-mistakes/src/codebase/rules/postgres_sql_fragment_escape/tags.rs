use crate::codebase::dependencies::extract::{
    ImportedBinding, ImportedBindingKind, ValueBinding, ValueCall, ValueExpression,
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
    ts.value_flow.tagged_templates.contains_key(&(start, end))
        && ts
            .value_flow
            .calls
            .iter()
            .find(|call| call.offset == start && call.end == end)
            .is_some_and(|call| call.arguments.len() == 1 && trusted_tag_call(ts, tags, call))
}

pub(super) fn has_tagged_query(ts: &TsFileFacts, tags: &[TrustedSqlTag]) -> bool {
    !ts.value_flow.tagged_templates.is_empty()
        && ts
            .value_flow
            .calls
            .iter()
            .any(|call| trusted_tag_call(ts, tags, call))
}

fn trusted_tag_call(ts: &TsFileFacts, tags: &[TrustedSqlTag], call: &ValueCall) -> bool {
    let ValueExpression::Named { name, scope, .. } = &call.callee else {
        return false;
    };
    scope.is_none_or(|scope| scope == 0)
        && ts
            .value_flow
            .tagged_templates
            .contains_key(&(call.offset, call.end))
        && binding_is_tag(ts, tags, name)
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
