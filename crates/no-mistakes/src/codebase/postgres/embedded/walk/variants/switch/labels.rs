use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrowFunctionExpression, AssignmentExpression, CallExpression, Expression, Function,
    UpdateExpression,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

/// Failed labels can change scalar guards before a later case or default runs.
/// The switch fallback discards finite facts instead of replaying those writes
/// on each entry/fallthrough path.
pub(super) fn contains_write(expression: &Expression<'_>) -> bool {
    let mut writes = Writes::default();
    writes.visit_expression(expression);
    writes.found
}

#[derive(Default)]
struct Writes {
    found: bool,
}

impl<'a> Visit<'a> for Writes {
    fn visit_assignment_expression(&mut self, _: &AssignmentExpression<'a>) {
        // This includes object/array targets and writes through members.
        self.found = true;
    }

    fn visit_update_expression(&mut self, _: &UpdateExpression<'a>) {
        self.found = true;
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        // Evaluate the callee and arguments before the invoked literal body.
        // Merely traversing a member's receiver still skips its closure body.
        walk::walk_call_expression(self, call);
        if self.found {
            return;
        }
        match invocation_target(&call.callee) {
            Expression::ArrowFunctionExpression(arrow) => {
                walk::walk_arrow_function_expression(self, arrow);
            }
            Expression::FunctionExpression(function) => {
                walk::walk_function(self, function, ScopeFlags::Function);
            }
            _ => {}
        }
    }

    // Creating or passing a callback does not execute its body.
    fn visit_function(&mut self, _: &Function<'a>, _: ScopeFlags) {}
    fn visit_arrow_function_expression(&mut self, _: &ArrowFunctionExpression<'a>) {}
}

fn invocation_target<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    let expression = unwrap_ts_wrappers(expression);
    let receiver = match expression {
        Expression::StaticMemberExpression(member)
            if matches!(member.property.name.as_str(), "call" | "apply") =>
        {
            &member.object
        }
        Expression::ComputedMemberExpression(member)
            if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(property)
                if matches!(property.value.as_str(), "call" | "apply")) =>
        {
            &member.object
        }
        _ => return expression,
    };
    unwrap_ts_wrappers(receiver)
}
