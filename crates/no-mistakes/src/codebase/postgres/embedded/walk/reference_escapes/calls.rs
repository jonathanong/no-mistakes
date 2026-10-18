use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::bindings::callee_name;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn begin_safe_builder_arguments(
        &mut self,
        call: &CallExpression<'_>,
        escaped: &[(String, Option<u32>)],
        modeled_builder: Option<&str>,
    ) -> usize {
        let start = self.builder_reference_exemptions.len();
        self.allow_copied_factory_arguments(call);
        match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                self.allow_builder_value(&member.object);
                for argument in call
                    .arguments
                    .iter()
                    .filter_map(|argument| argument.as_expression())
                {
                    self.allow_builder_value(argument);
                }
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(name) if name.value == "append") =>
            {
                self.allow_builder_value(&member.object);
                for argument in call
                    .arguments
                    .iter()
                    .filter_map(|argument| argument.as_expression())
                {
                    self.allow_builder_value(argument);
                }
            }
            _ => {}
        }
        let parameter = if self.executor_alias_callee(&call.callee)
            || callee_name(call, self.bindings, self.scoped).is_some()
        {
            Some(0)
        } else {
            match unwrap_ts_wrappers(&call.callee) {
                Expression::Identifier(id) if !self.shadowed_locally(id.name.as_str()) => self
                    .functions
                    .parameter_builder(id.name.as_str())
                    .map(|helper| helper.parameter_index),
                _ => None,
            }
        };
        if let Some(argument) = parameter
            .and_then(|index| call.arguments.get(index))
            .and_then(|argument| argument.as_expression())
        {
            self.allow_builder_value(argument);
        }
        if modeled_builder.is_some() || !escaped.is_empty() {
            for argument in call
                .arguments
                .iter()
                .filter_map(|argument| argument.as_expression())
            {
                if self.builder_result_names(argument).iter().any(|name| {
                    modeled_builder == Some(name.as_str())
                        || escaped.iter().any(|effect| effect.0 == *name)
                }) {
                    self.allow_builder_value(argument);
                }
            }
        }
        start
    }

    fn allow_copied_factory_arguments(&mut self, call: &CallExpression<'_>) {
        if !self.fresh_builder_call(call) {
            return;
        }
        for argument in call
            .arguments
            .iter()
            .filter_map(|argument| argument.as_expression())
        {
            self.allow_builder_value(argument);
        }
        // Only a trusted join copies a literal fragment list. Other containers,
        // including arrays handed to executors or unknown calls, still escape.
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            return;
        };
        if member.property.name != "join" {
            return;
        }
        if let Some(Expression::ArrayExpression(array)) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .map(unwrap_ts_wrappers)
        {
            for element in array
                .elements
                .iter()
                .filter_map(|element| element.as_expression())
            {
                self.allow_builder_value(element);
            }
        }
    }
}
