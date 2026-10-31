//! Map recovered placeholder indexes onto the template holes that occupy them.
use super::bindings::Bindings;
use super::identifier_positions::aligned;
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ValueExpression;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::fx::FxHashSet;
use std::path::Path;

use super::super::SqlCall;

pub(super) fn selected<'a>(
    facts: &'a CheckFactMap,
    call: &SqlCall<'a>,
    bindings: &Bindings<'a>,
) -> Option<Vec<&'a ValueExpression>> {
    let (indexes, max_index) = aligned(call.call)?;
    let value_call = facts
        .get_ts_facts(&call.file)?
        .value_flow
        .calls
        .iter()
        .find(|value_call| (value_call.offset, value_call.end) == call.span)?;
    let holes = template_holes(bindings, &call.file, value_call, max_index)?;
    if holes.len() != max_index || indexes.is_empty() {
        return None;
    }
    let mut selected = Vec::new();
    for index in indexes {
        let hole = holes.get(usize::try_from(index).ok()?.checked_sub(1)?)?;
        selected.push(*hole);
    }
    Some(selected)
}

fn template_holes<'a>(
    bindings: &Bindings<'a>,
    file: &Path,
    value_call: &'a crate::codebase::dependencies::extract::ValueCall,
    max_index: usize,
) -> Option<Vec<&'a ValueExpression>> {
    let arguments = value_call.arguments.as_slice();
    let first = arguments.first()?;
    let mut seen = FxHashSet::default();
    if let Some(parts) = template_parts(bindings, file, first, &mut seen) {
        return Some(parts);
    }
    if matches!(first, ValueExpression::Unknown) && arguments.len() == max_index.saturating_add(1) {
        return Some(arguments[1..].iter().collect());
    }
    None
}

fn template_parts<'a>(
    bindings: &Bindings<'a>,
    file: &Path,
    expr: &'a ValueExpression,
    seen: &mut FxHashSet<(usize, String, u32)>,
) -> Option<Vec<&'a ValueExpression>> {
    match expr {
        ValueExpression::Derived(parts) => Some(parts.iter().collect()),
        ValueExpression::Awaited(inner) => template_parts(bindings, file, inner, seen),
        ValueExpression::Named {
            name,
            scope,
            offset,
            ..
        } => {
            let binding = bindings.value(file, scope.unwrap_or(0), name, *offset)?;
            if !seen.insert((binding.scope, binding.name.clone(), binding.declared_at)) {
                return None;
            }
            template_parts(bindings, file, &binding.value, seen)
        }
        _ => None,
    }
}
