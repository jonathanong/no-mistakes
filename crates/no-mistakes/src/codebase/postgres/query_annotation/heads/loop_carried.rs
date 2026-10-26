//! Narrow backedge witnesses, separate from low's conservative write union.
use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
use std::collections::BTreeMap;
use std::sync::Arc;
pub(super) mod condition_key;
mod grammar;
mod path;
pub(super) use grammar::{iteration, test, update};
use grammar::{sequence, Flow};

pub(super) struct Candidate {
    call: Span,
    reference: Span,
    pub(super) assignment: Span,
    pub(super) tag: Option<Span>,
    loop_span: Span,
    path: Arc<path::Node>,
    pub(super) condition_references: Arc<Vec<Span>>,
}

/// Unconditional calls and literals form candidates; path checks below decide
/// which conditional writes and exits leave a live backedge.
pub(super) fn candidates(body: &Statement<'_>, loop_span: oxc_span::Span) -> Vec<Candidate> {
    let mut expressions = Vec::new();
    match sequence(body, &mut expressions) {
        Flow::Unsupported => return Vec::new(),
        Flow::Backedge | Flow::Exit | Flow::Next => {}
    }
    let path = Arc::new(path::Node::from_statement(body));
    let mut condition_references = Vec::new();
    path.condition_references(&mut condition_references);
    let condition_references = Arc::new(condition_references);
    let mut calls = crate::fx::FxHashMap::<
        &str,
        Vec<(&CallExpression<'_>, &IdentifierReference<'_>)>,
    >::default();
    let mut assignments = Vec::new();
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
                assignments.push((target.name.as_str(), assignment, target));
            }
        }
    }
    let mut candidates = Vec::new();
    for (name, assignment, target) in assignments {
        if let Some(prior) = calls.get(name) {
            candidates.extend(prior.iter().filter_map(|(call, reference)| {
                pair(
                    call,
                    reference,
                    assignment,
                    target,
                    loop_span,
                    &path,
                    &condition_references,
                )
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
    path: &Arc<path::Node>,
    condition_references: &Arc<Vec<Span>>,
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
        loop_span: (loop_span.start, loop_span.end),
        path: Arc::clone(path),
        condition_references: Arc::clone(condition_references),
    })
}

pub(super) fn proofs(
    syntax: &Syntax,
    bindings: &Bindings<'_>,
) -> BTreeMap<Span, Vec<Option<String>>> {
    let mut proofs = BTreeMap::<Span, Vec<Option<String>>>::new();
    for (call, tag) in syntax.loop_candidates.iter().filter_map(|candidate| {
        let owner = bindings.owner(candidate.reference)?;
        let target = bindings.owner(candidate.assignment)?;
        if (owner.span.start, owner.span.end) != (target.span.start, target.span.end)
            || owner.imported
            || owner.span.start >= candidate.loop_span.0
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
        let writes = owner
            .references
            .iter()
            .filter(|usage| usage.runtime)
            .filter_map(|usage| {
                let span = (usage.span.start, usage.span.end);
                (syntax.assignments.writes.contains_key(&span)
                    || syntax.assignments.unsupported_writes.contains(&span))
                .then_some(span)
            })
            .collect::<Vec<_>>();
        if !candidate.path.witness(
            &writes,
            candidate.assignment,
            candidate.reference,
            |condition| bindings.stable_condition(condition, candidate.loop_span, syntax),
        ) {
            return None;
        }
        // Keep the mutable call's function, executor, alias, and gap guards.
        // Its joined value may be unknown because a conditional reset exists;
        // the path proof above decides whether a literal still reaches it.
        if !bindings.valid_mutable_call(candidate.reference) {
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
    }) {
        let tags = proofs.entry(call).or_default();
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    proofs
}
