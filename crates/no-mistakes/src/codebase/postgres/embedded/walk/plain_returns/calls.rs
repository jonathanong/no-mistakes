use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::Expression;

pub(super) fn walk_calls(expression: &Expression<'_>, visit: &mut impl FnMut(&str)) {
    match unwrap_ts_wrappers(expression) {
        Expression::CallExpression(call) => {
            if let Expression::Identifier(ident) = unwrap_ts_wrappers(&call.callee) {
                visit(ident.name.as_str());
            }
            for argument in &call.arguments {
                if let Some(argument) = argument.as_expression() {
                    walk_calls(argument, visit);
                }
            }
        }
        Expression::ConditionalExpression(value) => {
            walk_calls(&value.consequent, visit);
            walk_calls(&value.alternate, visit);
        }
        Expression::LogicalExpression(value) => {
            walk_calls(&value.left, visit);
            walk_calls(&value.right, visit);
        }
        Expression::BinaryExpression(value) => {
            walk_calls(&value.left, visit);
            walk_calls(&value.right, visit);
        }
        Expression::SequenceExpression(value) => {
            for item in &value.expressions {
                walk_calls(item, visit);
            }
        }
        Expression::TemplateLiteral(value) => {
            for item in &value.expressions {
                walk_calls(item, visit);
            }
        }
        Expression::TaggedTemplateExpression(value) => {
            for item in &value.quasi.expressions {
                walk_calls(item, visit);
            }
        }
        _ => {}
    }
}
