use super::{binding_tag, value_call_at, Bindings, Provenance};
use crate::codebase::dependencies::extract::{ValueBinding, ValueExpression};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::postgres_sql_cross_file::SqlCall;
use crate::fx::FxHashSet;
use std::path::Path;

mod reads;

type Identity = (usize, String, u32);

pub(super) fn resolve<'a>(
    bindings: &Bindings<'a>,
    file: &Path,
    mut value: &'a ValueExpression,
) -> Option<&'a ValueExpression> {
    let mut seen = FxHashSet::default();
    while let ValueExpression::Named {
        name,
        scope,
        offset,
        ..
    } = value
    {
        let binding = bindings.value(file, scope.unwrap_or(0), name, *offset)?;
        if !binding.immutable || binding.invalidated_at.is_some() || !seen.insert(identity(binding))
        {
            return None;
        }
        value = &binding.value;
    }
    Some(value)
}

fn identity(binding: &ValueBinding) -> Identity {
    (binding.scope, binding.name.clone(), binding.declared_at)
}

fn owner<'a>(
    bindings: &Bindings<'a>,
    file: &Path,
    value: &ValueExpression,
) -> Option<&'a ValueBinding> {
    let mut value = value;
    let mut seen = FxHashSet::default();
    loop {
        let ValueExpression::Named {
            name,
            scope,
            offset,
            ..
        } = value
        else {
            return None;
        };
        let binding = bindings.value(file, scope.unwrap_or(0), name, *offset)?;
        if !binding.immutable || binding.invalidated_at.is_some() || !seen.insert(identity(binding))
        {
            return None;
        }
        if matches!(binding.value, ValueExpression::Named { .. }) {
            value = &binding.value;
        } else {
            return Some(binding);
        }
    }
}

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

pub(crate) fn detached_template_binds(
    call: &SqlCall<'_>,
    bindings: &Bindings<'_>,
) -> Option<Provenance> {
    let ts = bindings.facts.get_ts_facts(&call.file)?;
    let executor = ts
        .value_flow
        .calls
        .iter()
        .find(|value| (value.offset, value.end) == call.span)?;
    let first = executor.arguments.first()?;
    if !matches!(first, ValueExpression::Named { .. }) {
        return None;
    }
    let ValueExpression::Derived(parts) = resolve(bindings, &call.file, first)? else {
        return None;
    };
    let [ValueExpression::Member(base, property)] = parts.as_ref() else {
        return None;
    };
    if property != "text" {
        return None;
    }
    let binding = owner(bindings, &call.file, base)?;
    if binding.scope == 0
        && ts.exported_bindings.iter().any(|export| {
            export.specifier.is_none()
                && (export.local == binding.name
                    || ts.value_flow.bindings.iter().any(|exported| {
                        exported.scope == 0
                            && exported.name == export.local
                            && escapes(bindings, &call.file, &exported.value, binding)
                    }))
        })
        || ts
            .value_flow
            .returns
            .iter()
            .any(|(_, value)| escapes(bindings, &call.file, value, binding))
    {
        return None;
    }
    if ts
        .value_flow
        .identifier_writes
        .iter()
        .any(|write| write.scope == binding.scope && write.name == binding.name)
        || !reads::closed(bindings, &call.file, &ts.value_flow, binding)
    {
        return None;
    }
    if ts.value_flow.calls.iter().any(|value| {
        matches!(&value.callee, ValueExpression::Member(base, _)
            if owner(bindings, &call.file, base).is_some_and(|other| identity(other) == identity(binding)))
            || value.arguments.iter().any(|argument| {
                escapes(bindings, &call.file, argument, binding)
            })
    }) {
        return None;
    }
    let ValueExpression::Call(start, end) = resolve(bindings, &call.file, &binding.value)? else {
        return None;
    };
    let tag = value_call_at(bindings.facts, &call.file, *start, *end)?;
    if tag.arguments.len() <= 1 || !binding_tag::is_bind_tag(bindings, &call.file, tag) {
        return None;
    }
    let paired = executor.arguments.len() >= 2
        && executor.spread_at.is_none_or(|index| index > 1)
        && !executor.bound
        && executor.arguments.get(1).is_some_and(|value| {
            let Some(ValueExpression::Member(base, property)) =
                resolve(bindings, &call.file, value)
            else {
                return false;
            };
            property == "values"
                && owner(bindings, &call.file, base)
                    .is_some_and(|other| identity(other) == identity(binding))
        });
    (!paired).then(|| Provenance::Reference {
        name: binding.name.clone(),
        scope: binding.scope,
        offset: binding.declared_at,
        members: vec!["text".into()],
    })
}
