use super::*;
use oxc_ast::ast::{
    BindingPattern, Declaration, Program, Statement, VariableDeclaration, VariableDeclarationKind,
};
mod effects;
mod mutations;
pub(super) use mutations::invalidate;
use mutations::invalidate_statement;

pub(super) fn bind(pattern: &BindingPattern<'_>, value: Value, env: &mut Environment) {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => {
            env.insert(ident.name.to_string(), value);
        }
        BindingPattern::ArrayPattern(array) => {
            let values = if let Value::Array(values) = value {
                values
            } else {
                Arc::new(Vec::new())
            };
            for (index, element) in array.elements.iter().enumerate() {
                if let Some(pattern) = element {
                    bind(
                        pattern,
                        values.get(index).cloned().unwrap_or(Value::Unknown),
                        env,
                    );
                }
            }
            if let Some(rest) = &array.rest {
                bind(&rest.argument, Value::Unknown, env);
            }
        }
        BindingPattern::ObjectPattern(object) => {
            let values = if let Value::Object(values, true) = value {
                values
            } else {
                Arc::new(BTreeMap::new())
            };
            for property in &object.properties {
                let value = static_property_key_name(&property.key)
                    .and_then(|name| values.get(name))
                    .map(|(value, _)| value.clone())
                    .unwrap_or(Value::Unknown);
                bind(&property.value, value, env);
            }
            if let Some(rest) = &object.rest {
                bind(&rest.argument, Value::Unknown, env);
            }
        }
        BindingPattern::AssignmentPattern(pattern) => bind(&pattern.left, Value::Unknown, env),
    }
}

pub(super) fn declaration(
    evaluator: &mut Evaluator,
    var: &VariableDeclaration<'_>,
    env: &mut Environment,
) {
    for declarator in &var.declarations {
        let mut value = if var.kind == VariableDeclarationKind::Const {
            declarator
                .init
                .as_ref()
                .map(|expr| evaluator.expression(expr, env, 0))
                .unwrap_or(Value::Unknown)
        } else {
            Value::Unknown
        };
        if let Some(init) = &declarator.init {
            if effects::has_unknown_call(init) {
                invalidate(init, env);
                value = Value::Unknown;
            }
        }
        if matches!(value, Value::Unknown) {
            if let Some(init) = &declarator.init {
                invalidate(init, env);
            }
        }
        bind(&declarator.id, value, env);
    }
}

pub(in super::super) fn program_environment(program: &Program<'_>) -> Environment {
    let mut env = Environment::new();
    let mut evaluator = Evaluator::new();
    for statement in &program.body {
        match statement {
            Statement::VariableDeclaration(var) => declaration(&mut evaluator, var, &mut env),
            Statement::ExportDeclaration(export) => {
                if let Declaration::VariableDeclaration(var) = &export.declaration {
                    declaration(&mut evaluator, var, &mut env);
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let Some(expr) = export.declaration.as_expression() {
                    if effects::has_unknown_call(expr) {
                        invalidate(expr, &mut env);
                    }
                }
            }
            Statement::FunctionDeclaration(_) | Statement::ImportDeclaration(_) => {}
            _ => invalidate_statement(statement, &mut env),
        }
    }
    env
}

pub(in super::super) fn scope_environment(
    statements: &[Statement<'_>],
    outer: &Environment,
) -> Environment {
    let mut env = outer.clone();
    for statement in statements {
        if let Statement::VariableDeclaration(var) = statement {
            for declarator in &var.declarations {
                bind(&declarator.id, Value::Unknown, &mut env);
            }
        }
    }
    let mut evaluator = Evaluator::new();
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(var) => declaration(&mut evaluator, var, &mut env),
            Statement::FunctionDeclaration(_) => {}
            _ => mutations::invalidate_statement(statement, &mut env),
        }
    }
    env
}
