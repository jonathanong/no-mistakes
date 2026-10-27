use super::{Expr, Scope, Stmt};
use crate::codebase::postgres::embedded::helpers::program::Param;
use oxc_ast::ast::{BindingPattern, FormalParameters, PropertyKey};

pub(super) fn lower(
    scope: &mut Scope<'_>,
    params: &FormalParameters<'_>,
) -> Option<(Vec<Param>, Vec<Stmt>)> {
    let mut result = Vec::new();
    let mut prologue = Vec::new();
    for (index, param) in params.items.iter().enumerate() {
        let default = param
            .initializer
            .as_ref()
            .map(|value| scope.expression(value));
        let name = match &param.pattern {
            BindingPattern::BindingIdentifier(id) => scope.ctx.bindings.name(id.span)?,
            BindingPattern::ObjectPattern(pattern) if pattern.rest.is_none() => {
                // This namespace cannot collide with semantic binding slots.
                let slot = format!("argument:{index}");
                prologue.extend(destructure(scope, pattern, &slot)?);
                slot
            }
            _ => return None,
        };
        result.push(Param {
            name,
            builder: false,
            default,
        });
    }
    Some((result, prologue))
}

pub(super) fn destructure(
    scope: &mut Scope<'_>,
    pattern: &oxc_ast::ast::ObjectPattern<'_>,
    slot: &str,
) -> Option<Vec<Stmt>> {
    if pattern.rest.is_some() {
        return None;
    }
    let mut prologue = Vec::new();
    for property in &pattern.properties {
        if property.computed {
            return None;
        }
        let key = match &property.key {
            PropertyKey::StaticIdentifier(key) => key.name.to_string(),
            PropertyKey::StringLiteral(key) => crate::ast::js_literal(key).into_owned(),
            _ => return None,
        };
        if key == "__proto__" {
            return None;
        }
        let read = Expr::Member(Box::new(Expr::Ident(slot.to_string())), key);
        let (id, value) = match &property.value {
            BindingPattern::BindingIdentifier(id) => (id, read),
            BindingPattern::AssignmentPattern(assignment) => {
                let BindingPattern::BindingIdentifier(id) = &assignment.left else {
                    return None;
                };
                let fallback = scope.expression(&assignment.right);
                // Defaults run for undefined only, never for null.
                let test = Expr::Eq(
                    Box::new(read.clone()),
                    Box::new(Expr::Undefined),
                    true,
                    false,
                );
                (
                    id,
                    Expr::Cond(Box::new(test), Box::new(fallback), Box::new(read)),
                )
            }
            _ => return None,
        };
        prologue.push(Stmt::Let(scope.ctx.bindings.name(id.span)?, value));
    }
    Some(prologue)
}
