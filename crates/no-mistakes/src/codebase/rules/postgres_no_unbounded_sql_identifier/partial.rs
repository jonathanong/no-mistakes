use super::*;
use crate::codebase::dependencies::extract::{TextOrigin, TextOrigins};
use crate::codebase::rules::postgres_sql_cross_file::{identifier_status, CompiledOptions};
use crate::fx::FxHashSet;
use std::collections::BTreeSet;

#[path = "partial_mask.rs"]
mod projection;
use projection::mask;

pub(super) fn sources<'a>(
    graph: &DepGraph,
    opts: &CompiledOptions,
    facts: &'a CheckFactMap,
    call: &SqlCall<'_>,
    bindings: &Bindings<'a>,
) -> BTreeSet<Provenance> {
    match identifier_status(call.call) {
        Some(true) => return identifier_sources(graph, facts, call, bindings),
        Some(false) => return BTreeSet::new(),
        None => {}
    }
    let Some(ts) = facts.get_ts_facts(&call.file) else {
        return BTreeSet::new();
    };
    let Some(value) = ts
        .value_flow
        .calls
        .iter()
        .find(|value| (value.offset, value.end) == call.span)
        .and_then(|value| value.arguments.first())
    else {
        return BTreeSet::new();
    };
    let origins = ts.value_flow.identifier_text_origins.as_deref();
    let origin = origins.and_then(|origins| origins.argument(call.span, 0));
    let Some((holes, origin)) = holes(
        bindings,
        &call.file,
        value,
        origins,
        origin,
        &mut FxHashSet::default(),
    ) else {
        return BTreeSet::new();
    };
    let bounded: Vec<_> = holes
        .iter()
        .enumerate()
        .map(|(index, value)| {
            closed::hole_proven(
                graph,
                opts,
                facts,
                bindings,
                call,
                &ts.value_flow,
                (
                    value,
                    origin
                        .and_then(|origin| origin.parts.get(index))
                        .and_then(Option::as_ref),
                ),
            )
        })
        .collect();
    if !bounded.iter().any(|bounded| *bounded) {
        return BTreeSet::new();
    }
    let texts: Vec<_> = if call.call.variants.is_empty() {
        call.call.sql_text.iter().map(String::as_str).collect()
    } else {
        call.call
            .variants
            .iter()
            .map(|variant| variant.sql_text.as_str())
            .collect()
    };
    let mut sources = BTreeSet::new();
    for text in texts {
        let Some(text) = mask(text, &bounded) else {
            continue;
        };
        let recovered = crate::codebase::postgres::EmbeddedSqlCall {
            sql_text: Some(text),
            variants: Vec::new(),
            ..call.call.clone()
        };
        if scans_identifier(&recovered) {
            sources.extend(identifier_sources(
                graph,
                facts,
                &SqlCall {
                    file: call.file.clone(),
                    span: call.span,
                    call: &recovered,
                },
                bindings,
            ));
        }
    }
    sources
}

fn holes<'a>(
    bindings: &Bindings<'a>,
    file: &Path,
    value: &'a ValueExpression,
    origins: Option<&'a TextOrigins>,
    origin: Option<&'a TextOrigin>,
    seen: &mut FxHashSet<(usize, String, u32)>,
) -> Option<(&'a [ValueExpression], Option<&'a TextOrigin>)> {
    match value {
        ValueExpression::Derived(parts) => Some((parts, origin)),
        ValueExpression::Named {
            name,
            scope,
            offset,
            ..
        } => {
            let binding = bindings.value(file, scope.unwrap_or(0), name, *offset)?;
            if !binding.immutable
                || !seen.insert((binding.scope, binding.name.clone(), binding.declared_at))
            {
                return None;
            }
            let origin = origins.and_then(|origins| {
                origins.binding(binding.scope, &binding.name, binding.declared_at)
            });
            holes(bindings, file, &binding.value, origins, origin, seen)
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "partial_tests.rs"]
mod tests;
