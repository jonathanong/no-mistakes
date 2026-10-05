use super::*;
use oxc_ast_visit::Visit;
struct References<'e> {
    env: &'e mut Environment,
}
impl<'a> Visit<'a> for References<'_> {
    fn visit_identifier_reference(&mut self, ident: &oxc_ast::ast::IdentifierReference<'a>) {
        if let Some(value) = self.env.get(ident.name.as_str()).cloned() {
            for binding in self.env.values_mut() {
                if shares_value(binding, &value) {
                    *binding = Value::Unknown;
                }
            }
            self.env.insert(ident.name.to_string(), Value::Unknown);
        }
    }
    fn visit_function(&mut self, _: &oxc_ast::ast::Function<'a>, _: oxc_syntax::scope::ScopeFlags) {
    }
    fn visit_arrow_function_expression(&mut self, _: &oxc_ast::ast::ArrowFunctionExpression<'a>) {}
}
pub(in super::super) fn invalidate(expr: &Expression<'_>, env: &mut Environment) {
    References { env }.visit_expression(expr);
}
pub(super) fn invalidate_statement(statement: &Statement<'_>, env: &mut Environment) {
    References { env }.visit_statement(statement);
}
fn shares_value(value: &Value, target: &Value) -> bool {
    match (value, target) {
        (Value::Array(left), Value::Array(right)) if Arc::ptr_eq(left, right) => true,
        (Value::Object(left, _), Value::Object(right, _)) if Arc::ptr_eq(left, right) => true,
        (Value::Array(values), _) => values.iter().any(|value| shares_value(value, target)),
        (Value::Object(values, _), _) => values
            .values()
            .any(|(value, _)| shares_value(value, target)),
        _ => false,
    }
}
