use super::{contains_identity_writes, independent_argument, EffectIdentities, Targets};
use crate::codebase::postgres::embedded::walk::ScopeVisitor;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, CallExpression, Expression};

impl EffectIdentities {
    pub(super) fn call_value(
        &self,
        visitor: &ScopeVisitor<'_>,
        call: &CallExpression<'_>,
        expression: &Expression<'_>,
    ) -> Option<Targets> {
        if visitor.fresh_builder_call(call) {
            return visitor
                .builder_identity_for_init(expression)
                .map(|identity| vec![(String::new(), Some(identity))]);
        }
        let (argument, parameter_index) = match unwrap_ts_wrappers(&call.callee) {
            Expression::StaticMemberExpression(member) if member.property.name == "append" => {
                (&member.object, None)
            }
            Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(name) if name.value == "append") => {
                (&member.object, None)
            }
            Expression::Identifier(name) if !visitor.shadowed_locally(name.name.as_str()) => {
                let index = visitor
                    .functions
                    .parameter_builder(name.name.as_str())?
                    .parameter_index;
                (call.arguments.get(index)?.as_expression()?, Some(index))
            }
            _ => return None,
        };
        // The result retains the object captured at the receiver/argument's
        // evaluation time. Reading its name after later arguments can be stale.
        let surrounding_writes = call.arguments.iter().enumerate().any(|(index, value)| {
            Some(index) != parameter_index
                && match value {
                    Argument::SpreadElement(spread) => contains_identity_writes(&spread.argument),
                    value => contains_identity_writes(value.as_expression().unwrap()),
                }
        });
        if surrounding_writes && !independent_argument(visitor, argument) {
            return None;
        }
        self.value(visitor, argument)
    }
}
