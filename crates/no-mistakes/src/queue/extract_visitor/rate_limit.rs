use super::QueueVisitor;
use crate::queue::extract_model::RateLimitSite;
use crate::queue::source::line_number;
use oxc_ast::ast::{CallExpression, Expression, ThrowStatement};

impl QueueVisitor<'_> {
    pub(super) fn note_rate_limit_call(&mut self, call: &CallExpression<'_>) {
        self.note_rate_limit(call.span.start, is_rate_limit_call(call));
    }

    pub(super) fn note_rate_limit_throw(&mut self, stmt: &ThrowStatement<'_>) {
        self.note_rate_limit(stmt.span.start, is_rate_limit_throw(&stmt.argument));
    }

    fn note_rate_limit(&mut self, start: u32, matched: bool) {
        if !matched {
            return;
        }
        let Some(worker_index) = self.worker_stack.last().copied() else {
            return;
        };
        self.facts.rate_limits.push(RateLimitSite {
            line: line_number(self.source, start),
            worker_index,
        });
    }
}

fn is_rate_limit_call(call: &CallExpression<'_>) -> bool {
    let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression() else {
        return false;
    };
    member.property.name == "rateLimit"
}

fn is_rate_limit_throw(argument: &Expression<'_>) -> bool {
    let argument = argument.get_inner_expression();
    let callee = match argument {
        Expression::NewExpression(new_expr) => &new_expr.callee,
        Expression::CallExpression(call) => &call.callee,
        _ => return false,
    };
    callee_name(callee) == Some("RateLimitError")
}

fn callee_name<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    match expr.get_inner_expression() {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
        _ => None,
    }
}
