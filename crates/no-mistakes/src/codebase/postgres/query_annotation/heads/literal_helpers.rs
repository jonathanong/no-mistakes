//! Direct configured wrappers use their sole literal argument, never its aliases.
use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::{facts::TsFileFacts, unwrap_ts_wrappers};
use crate::fx::FxHashMap;
use oxc_ast::ast::{CallExpression, ChainElement, Expression};
use std::collections::BTreeMap;

pub(super) struct Candidate {
    pub reference: Span,
    span: Span,
    export: Option<String>,
    proof: HeadProof,
}

pub(super) fn candidate(outer: &CallExpression<'_>) -> Option<Candidate> {
    let argument = unwrap_ts_wrappers(outer.arguments.first()?.as_expression()?);
    let call = match argument {
        Expression::CallExpression(call) => call.as_ref(),
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::CallExpression(call) => call.as_ref(),
            _ => return None,
        },
        _ => return None,
    };
    let (reference, export, optional) = match unwrap_ts_wrappers(&call.callee) {
        Expression::Identifier(id) => (id.span, None, call.optional),
        Expression::StaticMemberExpression(member) => {
            let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) else {
                return None;
            };
            (
                id.span,
                Some(member.property.name.to_string()),
                call.optional || member.optional,
            )
        }
        Expression::ComputedMemberExpression(member) => {
            let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) else {
                return None;
            };
            (
                id.span,
                Some(literal_text(&member.expression)?.to_string()),
                call.optional || member.optional,
            )
        }
        _ => return None,
    };
    let text = match call.arguments.as_slice() {
        [argument] if !optional => argument.as_expression().and_then(literal_text),
        _ => None,
    };
    Some(Candidate {
        reference: (reference.start, reference.end),
        span: (outer.span.start, outer.span.end),
        export,
        proof: text.map_or(HeadProof::Unproven, |sql| {
            if crate::codebase::postgres::sql_requires_query_annotation(&sql) {
                HeadProof::Missing
            } else {
                HeadProof::Literal
            }
        }),
    })
}

// Share the exact literal grammar for SQL arguments and computed export keys.
fn literal_text<'a>(expression: &'a Expression<'a>) -> Option<std::borrow::Cow<'a, str>> {
    match unwrap_ts_wrappers(expression) {
        Expression::StringLiteral(literal) => Some(crate::ast::js_literal(literal)),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
            .quasis
            .first()
            .filter(|part| part.value.cooked.is_some())
            .map(crate::ast::template_quasi_text),
        _ => None,
    }
}

pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
    ts: &TsFileFacts,
) -> Option<BTreeMap<Span, (String, HeadProof)>> {
    let candidates = syntax.literal_helpers.as_ref()?;
    if let Some(observer) = crate::diagnostics::current() {
        observer.increment(
            "postgres.annotation.literal-helper.candidates",
            candidates.len() as u64,
        );
    }
    let module = ts.module_bindings.as_deref()?;
    let imports = module
        .imports
        .iter()
        .filter(|import| !import.type_only)
        .flat_map(|import| {
            import
                .bindings
                .iter()
                .filter(|binding| !binding.type_only)
                .map(move |binding| (binding.binding_id, (import.specifier.as_str(), binding)))
        })
        .collect::<FxHashMap<_, _>>();
    Some(
        candidates
            .iter()
            .filter_map(|candidate| {
                let owner = bindings
                    .owner(candidate.reference)
                    .filter(|owner| owner.imported)?;
                let (module, import) = imports.get(&owner.id)?;
                let export = match &candidate.export {
                    Some(export) if import.kind == "namespace" => export.as_str(),
                    None if import.kind != "namespace" => import.imported.as_str(),
                    _ => return None,
                };
                Some((
                    candidate.span,
                    (format!("{module}#{export}"), candidate.proof),
                ))
            })
            .collect(),
    )
}
