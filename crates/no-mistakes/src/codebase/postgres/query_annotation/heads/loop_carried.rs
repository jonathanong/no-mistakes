//! Narrow backedge witnesses, separate from low's conservative write union.
use super::{bindings::Bindings, syntax::Syntax, HeadProof, Span};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;
use std::collections::BTreeMap;

pub(super) struct Candidate {
    call: Span,
    reference: Span,
    pub(super) assignment: Span,
    pub(super) tag: Option<Span>,
    loop_start: u32,
}

/// The two-statement body has no reset, exit, or effect that can kill its backedge.
/// Other loop forms keep their existing conservative annotation behavior.
pub(super) fn candidate(value: &ForOfStatement<'_>) -> Option<Candidate> {
    if value.r#await || !matches!(&value.right, Expression::Identifier(_)) {
        return None;
    }
    let ForStatementLeft::VariableDeclaration(left) = &value.left else {
        return None;
    };
    // Successful for-of syntax always has exactly one declaration.
    let binding = &left.declarations[0];
    if !matches!(binding.id, BindingPattern::BindingIdentifier(_)) {
        return None;
    }
    let Statement::BlockStatement(body) = &value.body else {
        return None;
    };
    let [Statement::ExpressionStatement(first), Statement::ExpressionStatement(second)] =
        body.body.as_slice()
    else {
        return None;
    };
    let first = unwrap_ts_wrappers(&first.expression);
    let first = match first {
        Expression::AwaitExpression(value) => unwrap_ts_wrappers(&value.argument),
        value => value,
    };
    let Expression::CallExpression(call) = first else {
        return None;
    };
    // Canonical executor facts decide callee ownership, including member calls.
    if call.optional || call.arguments.len() != 1 {
        return None;
    }
    let Expression::Identifier(reference) = unwrap_ts_wrappers(call.arguments[0].as_expression()?)
    else {
        return None;
    };
    let Expression::AssignmentExpression(assignment) = unwrap_ts_wrappers(&second.expression)
    else {
        return None;
    };
    if assignment.operator != oxc_syntax::operator::AssignmentOperator::Assign {
        return None;
    }
    let AssignmentTarget::AssignmentTargetIdentifier(target) = &assignment.left else {
        return None;
    };
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
        loop_start: value.span.start,
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
