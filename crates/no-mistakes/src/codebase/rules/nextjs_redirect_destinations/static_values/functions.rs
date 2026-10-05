use super::{
    bindings::{bind, declaration, invalidate},
    *,
};
use oxc_ast::ast::{
    ArrowFunctionBody, ArrowFunctionExpression, CallExpression, FormalParameters, Function,
    FunctionBody, Statement,
};

impl Evaluator {
    pub(in super::super) fn function(
        &mut self,
        function: &Function<'_>,
        outer: &Environment,
    ) -> Value {
        let mut env = outer.clone();
        shadow_parameters(&function.params, &mut env);
        function
            .body
            .as_ref()
            .map(|body| self.body(body, env, 0))
            .unwrap_or(Value::Unknown)
    }
    pub(in super::super) fn arrow(
        &mut self,
        function: &ArrowFunctionExpression<'_>,
        outer: &Environment,
        depth: usize,
    ) -> Value {
        let mut env = outer.clone();
        shadow_parameters(&function.params, &mut env);
        self.arrow_body(&function.body, env, depth)
    }
    fn arrow_body(
        &mut self,
        body: &ArrowFunctionBody<'_>,
        env: Environment,
        depth: usize,
    ) -> Value {
        match body {
            ArrowFunctionBody::FunctionBody(body) => self.body(body, env, depth),
            _ => body
                .as_expression()
                .map(|expr| self.expression(expr, &env, depth))
                .unwrap_or(Value::Unknown),
        }
    }
    fn body(&mut self, body: &FunctionBody<'_>, mut env: Environment, depth: usize) -> Value {
        // Predeclare locals so a forward reference never falls back to an outer binding.
        for statement in &body.statements {
            if let Statement::VariableDeclaration(var) = statement {
                for declarator in &var.declarations {
                    bind(&declarator.id, Value::Unknown, &mut env);
                }
            }
        }
        for statement in &body.statements {
            match statement {
                Statement::VariableDeclaration(var) => declaration(self, var, &mut env),
                Statement::ReturnStatement(ret) => {
                    return ret
                        .argument
                        .as_ref()
                        .map(|expr| self.expression(expr, &env, depth))
                        .unwrap_or(Value::Unknown)
                }
                Statement::ExpressionStatement(expr) => invalidate(&expr.expression, &mut env),
                Statement::EmptyStatement(_) => {}
                _ => return Value::Unknown,
            }
        }
        Value::Array(Arc::new(Vec::new()))
    }
    pub(super) fn map(
        &mut self,
        call: &CallExpression<'_>,
        env: &Environment,
        depth: usize,
    ) -> Value {
        if call.optional || call.arguments.len() != 1 {
            return Value::Unknown;
        }
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            return Value::Unknown;
        };
        if member.optional || member.property.name != "map" {
            return Value::Unknown;
        }
        let Some(callback) = call.arguments[0].as_expression() else {
            return Value::Unknown;
        };
        let Expression::ArrowFunctionExpression(callback) = unwrap_ts_wrappers(callback) else {
            return Value::Unknown;
        };
        if callback.r#async || callback.params.items.len() != 1 || callback.params.rest.is_some() {
            return Value::Unknown;
        }
        let Value::Array(values) = self.expression(&member.object, env, depth) else {
            return Value::Unknown;
        };
        let mut output = Vec::new();
        for value in values.iter().cloned() {
            if self.remaining == 0 {
                output.push(Value::Unknown);
                break;
            }
            let mut local = env.clone();
            bind(&callback.params.items[0].pattern, value, &mut local);
            output.push(self.arrow_body(&callback.body, local, depth));
        }
        Value::Array(Arc::new(output))
    }
}
fn shadow_parameters(params: &FormalParameters<'_>, env: &mut Environment) {
    for parameter in &params.items {
        bind(&parameter.pattern, Value::Unknown, env);
    }
    if let Some(rest) = &params.rest {
        bind(&rest.rest.argument, Value::Unknown, env);
    }
}

pub(in super::super) fn parameter_environment(
    params: &FormalParameters<'_>,
    outer: &Environment,
) -> Environment {
    let mut env = outer.clone();
    shadow_parameters(params, &mut env);
    env
}
