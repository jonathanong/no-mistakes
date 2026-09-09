use super::super::super::placeholders::{count_placeholders, renumber_placeholders};
use super::super::super::EmbeddedSqlKind;
use super::compose::static_fragment;
use super::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, CallExpression, Expression};

pub(crate) fn apply_append(visitor: &mut ScopeVisitor<'_>, call: &CallExpression<'_>) {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return;
    };
    if member.property.name != "append" {
        return;
    }
    let Expression::Identifier(ident) = unwrap_ts_wrappers(&member.object) else {
        return;
    };
    let Some(arg) = first_static_arg(call, visitor) else {
        visitor.mark_dynamic(ident.name.as_str());
        return;
    };
    if visitor.control_depth > 0 {
        visitor.mark_dynamic(ident.name.as_str());
        return;
    }
    for scope in visitor.scopes.iter_mut().rev() {
        if let Some(binding) = scope.get_mut(ident.name.as_str()) {
            match (&binding.sql, binding.kind) {
                (Some(sql), EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed) => {
                    let arg = renumber_placeholders(&arg, count_placeholders(sql));
                    binding.sql = Some(format!("{sql}{arg}"));
                    binding.kind = EmbeddedSqlKind::Composed;
                }
                _ => {
                    binding.kind = EmbeddedSqlKind::Dynamic;
                    binding.sql = None;
                }
            }
            return;
        }
    }
}

fn first_static_arg(call: &CallExpression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let Argument::SpreadElement(_) = call.arguments.first()? else {
        let expr = call.arguments.first()?.as_expression()?;
        return static_fragment(expr, visitor);
    };
    None
}
