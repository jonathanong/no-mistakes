use super::{resolve, ScopeVisitor};
use crate::codebase::postgres::embedded::tags;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_ast_visit::walk;

/// Only executor ownership and a diagnostic head survive this projection.
/// The annotation owner extracts its complete head facts in the same AST pass.
pub(super) fn text(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let mut head = unwrap_ts_wrappers(expr);
    while let Expression::CallExpression(call) = head {
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            return None;
        };
        if member.property.name != "append" {
            return None;
        }
        head = unwrap_ts_wrappers(&member.object);
    }
    match head {
        Expression::Identifier(name) => visitor.lookup(name.name.as_str())?.sql,
        Expression::StringLiteral(value) => Some(crate::ast::js_literal(&value).into_owned()),
        Expression::TemplateLiteral(value) => {
            Some(tags::quasi_text(value.quasis.first()?, false).to_string())
        }
        Expression::TaggedTemplateExpression(value) => Some(
            tags::quasi_text(
                value.quasi.quasis.first()?,
                tags::is_string_raw_tag_spelling(&value.tag),
            )
            .to_string(),
        ),
        _ => None,
    }
}

impl<'a> ScopeVisitor<'a> {
    pub(super) fn visit_head_call(&mut self, call: &CallExpression<'a>) {
        let effect = self.call_builder_effect(call);
        let before = (self.calls.len(), self.pending_calls.len());
        resolve::record_executor_call(self, call);
        let arguments = self.begin_executor_arguments(call, before);
        let invocation = self.begin_argument_invocation(call);
        let exemptions = self.begin_safe_builder_arguments(call, &effect.0, None);
        walk::walk_call_expression(self, call);
        self.builder_reference_exemptions.truncate(exemptions);
        self.finish_argument_invocation(invocation);
        if let Some(arguments) = arguments {
            self.finish_executor_arguments(call, arguments);
        }
        self.apply_call_builder_effect(effect);
        self.revoke_namespace_arguments(&call.callee, &call.arguments);
    }
}
