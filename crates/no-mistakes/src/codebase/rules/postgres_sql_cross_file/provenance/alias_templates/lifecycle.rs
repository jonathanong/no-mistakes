use super::{binding_tag, identity, owner, reads, resolve, Bindings};
use crate::codebase::dependencies::extract::{ValueBinding, ValueExpression};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::FxHashMap;
use std::path::Path;

fn escapes(
    bindings: &Bindings<'_>,
    file: &Path,
    value: &ValueExpression,
    binding: &ValueBinding,
) -> bool {
    match value {
        ValueExpression::Named { .. } => {
            owner(bindings, file, value).is_some_and(|other| identity(other) == identity(binding))
        }
        ValueExpression::Object(fields, _) | ValueExpression::Array(fields) => fields
            .values()
            .any(|value| escapes(bindings, file, value, binding)),
        ValueExpression::Derived(parts) => parts
            .iter()
            .any(|value| escapes(bindings, file, value, binding)),
        ValueExpression::Awaited(inner) => escapes(bindings, file, inner, binding),
        ValueExpression::Either(left, right)
        | ValueExpression::Nullish(left, right)
        | ValueExpression::Default(left, right) => {
            escapes(bindings, file, left, binding) || escapes(bindings, file, right, binding)
        }
        _ => false,
    }
}

pub(super) fn eligible<'a>(
    bindings: &Bindings<'a>,
    file: &Path,
    ts: &TsFileFacts,
    binding: &'a ValueBinding,
    calls: &FxHashMap<(u32, u32), usize>,
) -> bool {
    if binding.scope == 0
        && ts.exported_bindings.iter().any(|export| {
            export.specifier.is_none()
                && (export.local == binding.name
                    || ts.value_flow.bindings.iter().any(|exported| {
                        exported.scope == 0
                            && exported.name == export.local
                            && escapes(bindings, file, &exported.value, binding)
                    }))
        })
        || ts
            .value_flow
            .returns
            .iter()
            .any(|(_, value)| escapes(bindings, file, value, binding))
    {
        return false;
    }
    if ts
        .value_flow
        .identifier_writes
        .iter()
        .any(|write| write.scope == binding.scope && write.name == binding.name)
        || !reads::closed(bindings, file, &ts.value_flow, binding)
    {
        return false;
    }
    if ts.value_flow.calls.iter().any(|value| {
        matches!(&value.callee, ValueExpression::Member(base, _)
            if owner(bindings, file, base).is_some_and(|other| identity(other) == identity(binding)))
            || value.arguments.iter().any(|argument| {
                escapes(bindings, file, argument, binding)
            })
    }) {
        return false;
    }
    let Some(ValueExpression::Call(start, end)) = resolve(bindings, file, &binding.value) else {
        return false;
    };
    calls.get(&(*start, *end)).is_some_and(|index| {
        let tag = &ts.value_flow.calls[*index];
        tag.arguments.len() > 1 && binding_tag::is_bind_tag(bindings, file, tag)
    })
}
