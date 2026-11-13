//! Statement-position effects, including a `forEach` callback whose expression body is discarded.
use super::Syntax;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, ExpressionStatement};

impl Syntax {
    pub(super) fn discarded_expression(&mut self, expression: &Expression<'_>) {
        if self.collect_helper_heads {
            let effects = super::super::effects::call(expression);
            self.effect_calls.extend(effects.into_iter().flatten());
        }
        let literal = super::super::append::reference(expression);
        if let Some(reference) = literal {
            self.discarded_appends.insert(reference);
        }
        if let Some(reference) = literal
            .is_none()
            .then(|| super::super::append::prefix_reference(expression))
            .flatten()
        {
            self.prefix_discarded_appends.insert(reference);
        }
    }

    /// `forEach` ignores the callback result, so a concise body is a discarded statement.
    pub(super) fn foreach_expression_body(&mut self, statement: &ExpressionStatement<'_>) {
        let Expression::CallExpression(call) = unwrap_ts_wrappers(&statement.expression) else {
            return;
        };
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            return;
        };
        if call.optional || member.optional || member.property.name != "forEach" {
            return;
        }
        let Some(Expression::ArrowFunctionExpression(arrow)) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .map(unwrap_ts_wrappers)
        else {
            return;
        };
        if let Some(expression) = arrow.body.as_expression() {
            self.discarded_expression(expression);
        }
    }
}
