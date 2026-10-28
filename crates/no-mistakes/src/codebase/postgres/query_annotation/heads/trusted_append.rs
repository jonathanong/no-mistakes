use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::module_facts::TypeScriptModuleFacts;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::Expression;
use std::collections::BTreeMap;

/// The exact tag reference beneath single-argument static append links. Primitive receivers,
/// String.raw, concatenation, helper calls and aliases never establish trust.
pub(super) fn tag<'a>(mut expression: &'a Expression<'a>) -> Option<Span> {
    loop {
        expression = unwrap_ts_wrappers(expression);
        match expression {
            Expression::CallExpression(call) => expression = super::append::prefix_receiver(call)?,
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

pub(super) fn import_ids(module: &TypeScriptModuleFacts) -> FxHashSet<usize> {
    module
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
        .collect()
}

pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    calls: &BTreeMap<Span, HeadProof>,
    candidates: &FxHashSet<Span>,
    low_candidates: Option<&FxHashSet<Span>>,
) -> (FxHashSet<Span>, FxHashMap<Span, HeadProof>) {
    if candidates.is_empty() && low_candidates.is_none_or(FxHashSet::is_empty) {
        return Default::default();
    }
    let mut legacy = FxHashSet::default();
    let mut prefix = FxHashMap::default();
    for call in &syntax.calls {
        let low = low_candidates.is_some_and(|spans| spans.contains(&call.span));
        let max = candidates.contains(&call.span);
        if !low && !max {
            continue;
        }
        let prefix_proof = low
            .then(|| bindings.prefix_candidate(call.head, call.span, false))
            .flatten()
            .filter(|proof| *proof != HeadProof::Unproven);
        let legacy_prefix_proof = max
            .then(|| bindings.prefix_candidate(call.head, call.span, true))
            .flatten()
            .filter(|proof| *proof != HeadProof::Unproven);
        if prefix_proof.is_none()
            && legacy_prefix_proof.is_none()
            && !matches!(calls.get(&call.span), Some(HeadProof::Literal))
        {
            continue;
        }
        let key = match call.head.reference() {
            Some(reference) => {
                let Some(owner) = bindings.owner(reference) else {
                    continue;
                };
                if owner.imported {
                    continue;
                }
                (owner.span.start, owner.span.end)
            }
            None => call.span,
        };
        let Some(tag) = syntax.append_tags.get(&key) else {
            continue;
        };
        let Some(owner) = bindings.owner(*tag) else {
            continue;
        };
        if !owner.imported || !bindings.trusted_import(owner.id) {
            continue;
        }
        if let Some(proof) = prefix_proof {
            prefix.insert(call.span, proof);
        }
        if max
            && legacy_prefix_proof.is_none()
            && matches!(calls.get(&call.span), Some(HeadProof::Literal))
            && matches!(
                bindings.proof_legacy(call.head, call.span),
                HeadProof::Literal
            )
        {
            legacy.insert(call.span);
        }
    }
    (legacy, prefix)
}
