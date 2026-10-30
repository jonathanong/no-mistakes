//! `process.env` mutations and where each one sits in its enclosing code.
use super::super::path_origin::{env_name, is_process_env, process_global};
use super::super::{EnvWrite, PathOrigin, PathProvenance, Projector};
use oxc_ast::ast::*;
use oxc_syntax::node::NodeId;

pub(super) fn from_assignment<'a>(
    projector: &mut Projector<'a, '_>,
    assignment: &AssignmentExpression<'a>,
    node: NodeId,
    out: &mut PathProvenance,
) {
    let semantic = projector.semantic;
    let name = match &assignment.left {
        AssignmentTarget::StaticMemberExpression(member) => {
            if member.property.name == "env" && process_global(semantic, &member.object) {
                out.env_unknown_write = true;
                return;
            }
            is_process_env(semantic, &member.object).then(|| member.property.name.to_string())
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            if !is_process_env(semantic, &member.object) {
                return;
            }
            match member.expression.get_inner_expression() {
                Expression::StringLiteral(key) => Some(crate::ast::js_literal(key).into_owned()),
                _ => {
                    out.env_unknown_write = true;
                    return;
                }
            }
        }
        _ => return,
    };
    let Some(name) = name else {
        return;
    };
    let origin = if assignment.operator == AssignmentOperator::Assign {
        projector.path_of(&assignment.right)
    } else {
        PathOrigin::unknown()
    };
    record(out, projector, node, name, origin, assignment.span.end);
}

pub(super) fn from_delete<'a>(
    projector: &mut Projector<'a, '_>,
    unary: &UnaryExpression<'a>,
    node: NodeId,
    out: &mut PathProvenance,
) {
    if unary.operator != oxc_syntax::operator::UnaryOperator::Delete {
        return;
    }
    let semantic = projector.semantic;
    let object = match unary.argument.get_inner_expression() {
        Expression::StaticMemberExpression(member) => &member.object,
        Expression::ComputedMemberExpression(member) => &member.object,
        _ => return,
    };
    if !is_process_env(semantic, object) {
        return;
    }
    match env_name(semantic, &unary.argument) {
        Some(name) => record(
            out,
            projector,
            node,
            name,
            PathOrigin::unknown(),
            unary.span.end,
        ),
        None => out.env_unknown_write = true,
    }
}

/// `vi.stubEnv(name, value)` and the `Object`/`Reflect` mutators applied to `process.env`.
pub(super) fn from_call<'a>(
    projector: &mut Projector<'a, '_>,
    call: &CallExpression<'a>,
    node: NodeId,
    out: &mut PathProvenance,
) {
    if super::stub_global::stubs_process(call) {
        out.env_unknown_write = true;
    }
    let semantic = projector.semantic;
    let Expression::StaticMemberExpression(callee) = call.callee.get_inner_expression() else {
        return;
    };
    let first = call.arguments.first().and_then(Argument::as_expression);
    if callee.property.name == "stubEnv" {
        let key = first.and_then(|first| match first.get_inner_expression() {
            Expression::StringLiteral(key) => Some(crate::ast::js_literal(key).into_owned()),
            _ => None,
        });
        match key {
            Some(name) => {
                let origin = call
                    .arguments
                    .get(1)
                    .and_then(Argument::as_expression)
                    .map_or_else(PathOrigin::unknown, |value| projector.path_of(value));
                record(out, projector, node, name, origin, call.span.end);
            }
            None => out.env_unknown_write = true,
        }
        return;
    }
    let mutator = matches!(&callee.object, Expression::Identifier(object)
    if matches!(object.name.as_str(), "Object" | "Reflect")
        && matches!(
            callee.property.name.as_str(),
            "assign" | "defineProperty" | "defineProperties" | "set" | "deleteProperty" | "setPrototypeOf"
        ));
    if mutator && first.is_some_and(|first| is_process_env(semantic, first)) {
        out.env_unknown_write = true;
    }
}

/// `const env = process.env` hands out a mutable alias this pass cannot follow.
pub(super) fn from_alias<'a>(
    projector: &mut Projector<'a, '_>,
    declarator: &VariableDeclarator<'a>,
    out: &mut PathProvenance,
) {
    if matches!(declarator.id, BindingPattern::BindingIdentifier(_))
        && declarator
            .init
            .as_ref()
            .is_some_and(|init| is_process_env(projector.semantic, init))
    {
        out.env_unknown_write = true;
    }
}

fn record(
    out: &mut PathProvenance,
    projector: &Projector<'_, '_>,
    node: NodeId,
    name: String,
    origin: PathOrigin,
    end: u32,
) {
    out.env_writes.push(EnvWrite {
        name,
        end,
        origin,
        scope: super::scope::write_scope(projector.semantic, node),
    });
}
