use super::*;
use oxc_ast_visit::Visit;
struct References<'e> {
    env: &'e mut Environment,
}
impl<'a> Visit<'a> for References<'_> {
    fn visit_call_expression(&mut self, call: &oxc_ast::ast::CallExpression<'a>) {
        // Unknown calls may run callbacks or closures that mutate any captured container.
        invalidate_containers(self.env);
        oxc_ast_visit::walk::walk_call_expression(self, call);
    }
    fn visit_new_expression(&mut self, expr: &oxc_ast::ast::NewExpression<'a>) {
        invalidate_containers(self.env);
        oxc_ast_visit::walk::walk_new_expression(self, expr);
    }
    fn visit_tagged_template_expression(
        &mut self,
        expr: &oxc_ast::ast::TaggedTemplateExpression<'a>,
    ) {
        invalidate_containers(self.env);
        oxc_ast_visit::walk::walk_tagged_template_expression(self, expr);
    }
    fn visit_variable_declaration(&mut self, declaration: &oxc_ast::ast::VariableDeclaration<'a>) {
        for item in &declaration.declarations {
            if let Some(init) = &item.init {
                if super::effects::has_unknown_call(init) {
                    self.visit_expression(init);
                }
            }
        }
    }
    fn visit_return_statement(&mut self, statement: &oxc_ast::ast::ReturnStatement<'a>) {
        if let Some(expr) = &statement.argument {
            if super::effects::has_unknown_call(expr) {
                self.visit_expression(expr);
            }
        }
    }
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
    shares_bounded(value, target, &mut 4096, 0)
}
fn shares_bounded(value: &Value, target: &Value, remaining: &mut usize, depth: usize) -> bool {
    // Exhaustion conservatively treats the binding as a possible alias.
    if *remaining == 0 || depth >= 64 {
        return true;
    }
    *remaining -= 1;
    match (value, target) {
        (Value::Array(left), Value::Array(right)) if Arc::ptr_eq(left, right) => true,
        (Value::Object(left, _), Value::Object(right, _)) if Arc::ptr_eq(left, right) => true,
        (Value::Array(values), _) => values
            .iter()
            .any(|value| shares_bounded(value, target, remaining, depth + 1)),
        (Value::Object(values, _), _) => values
            .values()
            .any(|(value, _)| shares_bounded(value, target, remaining, depth + 1)),
        _ => false,
    }
}

fn invalidate_containers(env: &mut Environment) {
    for binding in env.values_mut() {
        if matches!(binding, Value::Array(_) | Value::Object(_, _)) {
            *binding = Value::Unknown;
        }
    }
}
