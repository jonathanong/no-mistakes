use oxc_ast::ast::Expression;

#[cfg(test)]
use oxc_ast::ast::Program;
#[cfg(test)]
use oxc_ast_visit::{walk, Visit};
#[cfg(test)]
use oxc_span::Span;

#[cfg(test)]
struct StateVisitor {
    has_state: bool,
    span: Span,
}

#[cfg(test)]
fn within(node_span: Span, component_span: Span) -> bool {
    node_span.start >= component_span.start && node_span.end <= component_span.end
}

fn hook_callee_name(expr: &oxc_ast::ast::CallExpression<'_>) -> Option<String> {
    match &expr.callee {
        Expression::Identifier(id) => Some(id.name.as_ref().to_string()),
        Expression::StaticMemberExpression(m) => {
            if matches!(&m.object, Expression::Identifier(id) if id.name == "React") {
                Some(m.property.name.as_ref().to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(crate) fn call_sets_state(expr: &oxc_ast::ast::CallExpression<'_>) -> bool {
    hook_callee_name(expr).is_some_and(|name| {
        matches!(
            name.as_str(),
            "useState" | "useReducer" | "useOptimistic" | "useSyncExternalStore"
        )
    })
}

pub(crate) fn member_is_this_state(expr: &oxc_ast::ast::StaticMemberExpression<'_>) -> bool {
    matches!(&expr.object, Expression::ThisExpression(_))
        && matches!(expr.property.name.as_ref(), "state" | "setState")
}

#[cfg(test)]
impl<'a> Visit<'a> for StateVisitor {
    fn visit_call_expression(&mut self, expr: &oxc_ast::ast::CallExpression<'a>) {
        if !within(expr.span, self.span) {
            return;
        }
        if call_sets_state(expr) {
            self.has_state = true;
        }
        walk::walk_call_expression(self, expr);
    }

    fn visit_static_member_expression(&mut self, expr: &oxc_ast::ast::StaticMemberExpression<'a>) {
        if !within(expr.span, self.span) {
            return;
        }
        if member_is_this_state(expr) {
            self.has_state = true;
        }
        walk::walk_static_member_expression(self, expr);
    }
}

#[cfg(test)]
pub(crate) fn detect_has_state(program: &Program<'_>, span: Span) -> bool {
    let mut visitor = StateVisitor {
        has_state: false,
        span,
    };
    visitor.visit_program(program);
    visitor.has_state
}

#[cfg(test)]
mod tests;
