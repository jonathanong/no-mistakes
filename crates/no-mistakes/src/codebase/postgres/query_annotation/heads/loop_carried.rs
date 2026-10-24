//! Narrow backedge witnesses, separate from low's conservative write union.
use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
use std::collections::BTreeMap;
mod grammar;
pub(super) use grammar::{iteration, test, update};
use grammar::{sequence, Flow};

pub(super) struct Candidate {
    call: Span,
    reference: Span,
    pub(super) assignment: Span,
    pub(super) tag: Option<Span>,
    loop_start: u32,
    loop_end: u32,
}

/// Straight-line statements preserve a backedge. Harmless branches can remain
/// between witnesses; binding checks below reject all intervening resets.
pub(super) fn candidates(body: &Statement<'_>, loop_span: oxc_span::Span) -> Vec<Candidate> {
    let mut expressions = Vec::new();
    let mut loop_span = loop_span;
    match sequence(body, &mut expressions) {
        Flow::Unsupported => return Vec::new(),
        // Dead statements after an unconditional continue cannot reset the head.
        Flow::Backedge(end) => loop_span.end = end,
        Flow::Next => {}
    }
    let mut calls = crate::fx::FxHashMap::<
        &str,
        Vec<(&CallExpression<'_>, &IdentifierReference<'_>)>,
    >::default();
    let mut assignments = crate::fx::FxHashMap::default();
    for expression in expressions {
        let value = unwrap_ts_wrappers(expression);
        let value = match value {
            Expression::AwaitExpression(value) => unwrap_ts_wrappers(&value.argument),
            value => value,
        };
        if let Expression::CallExpression(call) = value {
            if let Some(Expression::Identifier(reference)) = call
                .arguments
                .first()
                .and_then(|value| value.as_expression())
                .map(unwrap_ts_wrappers)
            {
                calls
                    .entry(reference.name.as_str())
                    .or_default()
                    .push((call, reference));
            }
        }
        if let Expression::AssignmentExpression(assignment) = value {
            if let AssignmentTarget::AssignmentTargetIdentifier(target) = &assignment.left {
                assignments.insert(target.name.as_str(), (assignment, target));
            }
        }
    }
    let mut candidates = Vec::new();
    for (name, (assignment, target)) in assignments {
        if let Some(prior) = calls.get(name) {
            candidates.extend(prior.iter().filter_map(|(call, reference)| {
                pair(call, reference, assignment, target, loop_span)
            }));
        }
    }
    candidates
}

fn pair(
    call: &CallExpression<'_>,
    reference: &IdentifierReference<'_>,
    assignment: &AssignmentExpression<'_>,
    target: &IdentifierReference<'_>,
    loop_span: oxc_span::Span,
) -> Option<Candidate> {
    // Canonical executor facts decide callee ownership, including member calls.
    if call.optional
        || call.span.end >= assignment.span.start
        || assignment.operator != oxc_syntax::operator::AssignmentOperator::Assign
    {
        return None;
    }
    let expression = unwrap_ts_wrappers(&assignment.right);
    let tag = match expression {
        Expression::StringLiteral(_) => None,
        Expression::TemplateLiteral(value) if value.expressions.is_empty() => None,
        Expression::TaggedTemplateExpression(value) if value.quasi.expressions.is_empty() => {
            let Expression::Identifier(tag) = unwrap_ts_wrappers(&value.tag) else {
                return None;
            };
            Some((tag.span.start, tag.span.end))
        }
        _ => return None,
    };
    if super::syntax::head(expression).proof(false) != HeadProof::Missing {
        return None;
    }
    Some(Candidate {
        call: (call.span.start, call.span.end),
        reference: (reference.span.start, reference.span.end),
        assignment: (target.span.start, target.span.end),
        tag,
        loop_start: loop_span.start,
        loop_end: loop_span.end,
    })
}

pub(super) fn proofs(syntax: &Syntax, bindings: &Bindings<'_>) -> BTreeMap<Span, Option<String>> {
    syntax
        .loop_candidates
        .iter()
        .filter_map(|candidate| {
            let owner = bindings.owner(candidate.reference)?;
            let target = bindings.owner(candidate.assignment)?;
            if (owner.span.start, owner.span.end) != (target.span.start, target.span.end)
                || owner.imported
                || owner.span.start >= candidate.loop_start
            {
                return None;
            }
            if !syntax
                .assignments
                .declarations
                .contains_key(&(owner.span.start, owner.span.end))
            {
                return None;
            }
            // Earlier writes after the call are overwritten by the final literal.
            // A reset after that literal or before the next read kills this proof.
            if owner
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .any(|usage| {
                    let span = (usage.span.start, usage.span.end);
                    (span.0 < candidate.reference.0 || span.0 > candidate.assignment.0)
                        && span.0 >= candidate.loop_start
                        && span.1 <= candidate.loop_end
                        && (syntax.assignments.writes.contains_key(&span)
                            || syntax.assignments.unsupported_writes.contains(&span))
                })
            {
                return None;
            }
            // Require an existing mutable local proof: it checks function ownership,
            // executor identity, unsupported writes, aliases and ownership gaps.
            if bindings.proof(
                super::syntax::Head::Reference(candidate.reference),
                candidate.call,
            ) != HeadProof::Missing
            {
                return None;
            }
            let tag = match candidate.tag {
                Some(span) => {
                    let owner = bindings.owner(span)?;
                    if !owner.imported {
                        return None;
                    }
                    Some(owner.name.clone())
                }
                None => None,
            };
            Some((candidate.call, tag))
        })
        .collect()
}
