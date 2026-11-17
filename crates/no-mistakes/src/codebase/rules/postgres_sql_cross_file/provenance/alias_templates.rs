use super::{binding_tag, Bindings, Provenance};
use crate::codebase::dependencies::extract::{ValueBinding, ValueExpression};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::postgres_sql_cross_file::SqlCall;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

mod lifecycle;
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

#[derive(Default)]
struct File {
    calls: FxHashMap<(u32, u32), usize>,
    eligible: FxHashMap<Identity, bool>,
}

pub(crate) struct DetachedTemplateBinds<'a, 'facts> {
    bindings: &'a Bindings<'facts>,
    files: FxHashMap<PathBuf, File>,
}

impl<'a, 'facts> DetachedTemplateBinds<'a, 'facts> {
    pub(crate) fn new(bindings: &'a Bindings<'facts>) -> Self {
        Self {
            bindings,
            files: FxHashMap::default(),
        }
    }

    pub(crate) fn detached(&mut self, call: &SqlCall<'_>) -> Option<Provenance> {
        let bindings = self.bindings;
        let ts = bindings.facts.get_ts_facts(&call.file)?;
        let file = self.files.entry(call.file.clone()).or_insert_with(|| File {
            calls: ts.value_flow.calls.iter().enumerate().fold(
                FxHashMap::default(),
                |mut calls, (index, value)| {
                    calls.entry((value.offset, value.end)).or_insert(index);
                    calls
                },
            ),
            ..Default::default()
        });
        let executor = &ts.value_flow.calls[*file.calls.get(&call.span)?];
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
        let calls = &file.calls;
        if !*file
            .eligible
            .entry(identity(binding))
            .or_insert_with(|| lifecycle::eligible(bindings, &call.file, ts, binding, calls))
        {
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
}
