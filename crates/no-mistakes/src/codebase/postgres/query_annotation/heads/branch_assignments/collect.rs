//! Only a direct function-body `let; if/else; executor` triple is admitted.
use super::{Arm, Candidate};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::*;

pub(super) fn candidates(body: &FunctionBody<'_>) -> Vec<Candidate> {
    body.statements
        .windows(3)
        .filter_map(|statements| {
            let Statement::VariableDeclaration(declaration) = &statements[0] else {
                return None;
            };
            if declaration.kind != VariableDeclarationKind::Let
                || declaration.declarations.len() != 1
            {
                return None;
            }
            let declaration = &declaration.declarations[0];
            if declaration.init.is_some() {
                return None;
            }
            let BindingPattern::BindingIdentifier(binding) = &declaration.id else {
                return None;
            };
            let Statement::IfStatement(branch) = &statements[1] else {
                return None;
            };
            let arms = [arm(&branch.consequent)?, arm(branch.alternate.as_ref()?)?];
            let call = executor(&statements[2])?;
            if call.optional
                || call.arguments.len() != 1
                || !matches!(unwrap_ts_wrappers(&call.callee), Expression::Identifier(_))
            {
                return None;
            }
            let Expression::Identifier(reference) =
                unwrap_ts_wrappers(call.arguments[0].as_expression()?)
            else {
                return None;
            };
            Some(Candidate {
                body: (body.span.start, body.span.end),
                declaration: (binding.span.start, binding.span.end),
                arms,
                call: (call.span.start, call.span.end),
                reference: (reference.span.start, reference.span.end),
            })
        })
        .collect()
}
fn arm(statement: &Statement<'_>) -> Option<Arm> {
    let Statement::BlockStatement(block) = statement else {
        return None;
    };
    let [Statement::ExpressionStatement(statement)] = block.body.as_slice() else {
        return None;
    };
    let Expression::AssignmentExpression(assignment) = unwrap_ts_wrappers(&statement.expression)
    else {
        return None;
    };
    if assignment.operator != AssignmentOperator::Assign {
        return None;
    }
    let AssignmentTarget::AssignmentTargetIdentifier(reference) = &assignment.left else {
        return None;
    };
    let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(&assignment.right) else {
        return None;
    };
    let Expression::Identifier(tag) = unwrap_ts_wrappers(&tagged.tag) else {
        return None;
    };
    Some(Arm {
        reference: (reference.span.start, reference.span.end),
        tag: (tag.span.start, tag.span.end),
        proof: super::super::syntax::head(&assignment.right).proof(false),
    })
}
fn executor<'a>(statement: &'a Statement<'a>) -> Option<&'a CallExpression<'a>> {
    let expression = match statement {
        Statement::ExpressionStatement(statement) => &statement.expression,
        Statement::VariableDeclaration(declaration) if declaration.declarations.len() == 1 => {
            declaration.declarations[0].init.as_ref()?
        }
        _ => return None,
    };
    let expression = unwrap_ts_wrappers(expression);
    let expression = match expression {
        Expression::AwaitExpression(value) => unwrap_ts_wrappers(&value.argument),
        _ => expression,
    };
    match expression {
        Expression::CallExpression(call) => Some(call),
        _ => None,
    }
}
