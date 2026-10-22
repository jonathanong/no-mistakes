use oxc_ast::ast::{Argument, CallExpression, Declaration, Expression, Statement, TemplateLiteral};
use std::collections::HashSet;

/// Only a direct top-level `require("literal")` is accepted. A condition,
/// sequence, `void`, or nested call is a different shape and stays rejected.
pub(super) fn note_statement(spans: &mut HashSet<u32>, statement: &Statement<'_>) {
    match statement {
        Statement::ExpressionStatement(statement) => note_expr(spans, &statement.expression),
        Statement::VariableDeclaration(declaration) => note_variables(spans, declaration),
        Statement::ExportDefaultDeclaration(declaration) => {
            if let Some(expression) = declaration.declaration.as_expression() {
                note_expr(spans, expression);
            }
        }
        Statement::ExportDeclaration(declaration) => {
            if let Declaration::VariableDeclaration(declaration) = &declaration.declaration {
                note_variables(spans, declaration);
            }
        }
        _ => {}
    }
}

fn note_variables(spans: &mut HashSet<u32>, declaration: &oxc_ast::ast::VariableDeclaration<'_>) {
    for declarator in &declaration.declarations {
        if let Some(init) = &declarator.init {
            note_expr(spans, init);
        }
    }
}

fn note_expr(spans: &mut HashSet<u32>, expression: &Expression<'_>) {
    let Expression::CallExpression(call) =
        crate::codebase::ts_source::unwrap_ts_wrappers(expression)
    else {
        return;
    };
    if !call.optional && is_require(&call.callee) {
        spans.insert(call.span.start);
    }
}

pub(super) fn mock_method<'a>(call: &'a CallExpression<'_>) -> Option<&'a str> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    let Expression::Identifier(object) = &member.object else {
        return None;
    };
    if !matches!(object.name.as_str(), "vi" | "jest") {
        return None;
    }
    Some(member.property.name.as_str())
}

pub(super) fn rejected_mock(method: &str) -> Option<&'static str> {
    Some(match method {
        "doMock" => "doMock",
        "setMock" => "setMock",
        "unstable_mockModule" => "unstable_mockModule",
        _ => return None,
    })
}

pub(super) fn is_require(callee: &Expression<'_>) -> bool {
    matches!(callee, Expression::Identifier(identifier) if identifier.name == "require")
}

pub(super) fn mock_specifier(call: &CallExpression<'_>) -> Option<String> {
    let first = call.arguments.first()?;
    if let Some(specifier) = literal_arg(first) {
        return Some(specifier);
    }
    let Argument::ImportExpression(import) = first else {
        return None;
    };
    literal_expr(crate::codebase::ts_source::unwrap_ts_wrappers(
        &import.source,
    ))
}

pub(super) fn literal_arg(argument: &Argument<'_>) -> Option<String> {
    argument.as_expression().and_then(|expression| {
        literal_expr(crate::codebase::ts_source::unwrap_ts_wrappers(expression))
    })
}

pub(super) fn literal_expr(expression: &Expression<'_>) -> Option<String> {
    match expression {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) => static_template(template),
        _ => None,
    }
}

fn static_template(template: &TemplateLiteral<'_>) -> Option<String> {
    if !template.expressions.is_empty() {
        return None;
    }
    let mut value = String::new();
    for quasi in &template.quasis {
        value.push_str(quasi.value.cooked.as_ref().unwrap_or(&quasi.value.raw));
    }
    Some(value)
}
