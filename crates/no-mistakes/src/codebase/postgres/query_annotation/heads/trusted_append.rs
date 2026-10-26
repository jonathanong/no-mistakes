use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::{facts::TsFileFacts, unwrap_ts_wrappers};
use crate::fx::FxHashSet;
use oxc_ast::ast::Expression;
use std::collections::BTreeMap;

/// The exact tag reference beneath static append links. Primitive receivers,
/// String.raw, concatenation, helper calls and aliases never establish trust.
pub(super) fn tag<'a>(mut expression: &'a Expression<'a>) -> Option<Span> {
    loop {
        expression = unwrap_ts_wrappers(expression);
        match expression {
            Expression::CallExpression(call) => expression = super::append::literal_receiver(call)?,
            Expression::TaggedTemplateExpression(value) => {
                let Expression::Identifier(tag) = unwrap_ts_wrappers(&value.tag) else {
                    return None;
                };
                return Some((tag.span.start, tag.span.end));
            }
            _ => return None,
        }
    }
}

pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    calls: &BTreeMap<Span, HeadProof>,
    ts: &TsFileFacts,
    candidates: &FxHashSet<Span>,
) -> FxHashSet<Span> {
    if candidates.is_empty() {
        return Default::default();
    }
    let Some(module) = ts.module_bindings.as_deref() else {
        return Default::default();
    };
    let trusted = module
        .imports
        .iter()
        .filter(|import| import.specifier == "sql-template-strings" && !import.type_only)
        .flat_map(|import| &import.bindings)
        .filter(|binding| {
            !binding.type_only
                && (binding.imported == "default"
                    || (binding.imported == "sql" && binding.local.eq_ignore_ascii_case("sql")))
        })
        .map(|binding| binding.binding_id)
        .collect::<FxHashSet<_>>();
    syntax
        .calls
        .iter()
        .filter_map(|call| {
            if !candidates.contains(&call.span)
                || !matches!(calls.get(&call.span), Some(HeadProof::Literal))
            {
                return None;
            }
            let key = match call.head.reference() {
                Some(reference) => {
                    let owner = bindings.owner(reference)?;
                    if owner.imported {
                        return None;
                    }
                    (owner.span.start, owner.span.end)
                }
                None => call.span,
            };
            let owner = bindings.owner(*syntax.append_tags.get(&key)?)?;
            (owner.imported && trusted.contains(&owner.id)).then_some(call.span)
        })
        .collect()
}
