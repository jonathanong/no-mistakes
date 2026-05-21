use super::{is_client_method_name, modules::is_client_named_callee, ClientHttpVisitor};
use oxc_ast::ast::{Expression, MemberExpression};

impl ClientHttpVisitor<'_> {
    pub(super) fn client_call_expr(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::Identifier(id) => {
                let name = id.name.as_str();
                self.is_client_callee_name(name) || self.is_client_name(name)
            }
            Expression::ParenthesizedExpression(expr) => self.client_call_expr(&expr.expression),
            _ => expr
                .as_member_expression()
                .is_some_and(|member| self.client_call_member(member)),
        }
    }

    fn client_call_member(&self, member: &MemberExpression<'_>) -> bool {
        member.static_property_name().is_some_and(|name| {
            is_client_method_name(name) || is_client_named_callee("undici", name)
        }) && self.client_expr(member.object())
    }
}
