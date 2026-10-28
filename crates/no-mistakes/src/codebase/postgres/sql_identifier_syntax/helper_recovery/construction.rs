//! Constructor calls remain visible to bounded abrupt-effect admission.
use super::{lower, span, CallModel, FileModels};
use oxc_ast::ast::{Argument, NewExpression};

impl FileModels {
    pub fn construct(&mut self, call: &NewExpression<'_>, ctx: &lower::Context<'_>) {
        let chain = super::super::chain(&call.callee);
        let callee = chain
            .as_ref()
            .and_then(|(at, _)| ctx.bindings.id(oxc_span::Span::new(at.0, at.1)));
        self.calls.push(CallModel {
            span: span(call.span),
            construction: true,
            callee,
            members: chain.map_or(Vec::new(), |(_, parts)| parts.into_iter().skip(1).collect()),
            arguments: call
                .arguments
                .iter()
                .filter_map(Argument::as_expression)
                .map(|argument| lower::expression(argument, ctx))
                .collect(),
            caller: self.stack.last().copied(),
        });
    }
}
