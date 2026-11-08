use super::super::super::TypeScriptModuleFacts;
use super::super::{path_origin::env_name, PathProvenance};
use oxc_ast::{ast::*, AstKind};
use oxc_semantic::Semantic;
use oxc_syntax::node::NodeId;

pub(super) fn test_calls(
    semantic: &Semantic<'_>,
    facts: &TypeScriptModuleFacts,
    registration: &CallExpression<'_>,
    out: &mut PathProvenance,
) {
    for argument in &registration.arguments {
        let Some(expression) = argument.as_expression() else {
            continue;
        };
        let statements = match expression.get_inner_expression() {
            Expression::ArrowFunctionExpression(function) => match &function.body {
                ArrowFunctionBody::FunctionBody(body) => &body.statements,
                _ => continue,
            },
            Expression::FunctionExpression(function) => {
                let Some(body) = &function.body else { continue };
                &body.statements
            }
            _ => continue,
        };
        for pair in statements.windows(2) {
            let [Statement::ExpressionStatement(write), Statement::ExpressionStatement(read)] =
                pair
            else {
                continue;
            };
            let Expression::CallExpression(stub) = write.expression.get_inner_expression() else {
                continue;
            };
            let Expression::StaticMemberExpression(member) = stub.callee.get_inner_expression()
            else {
                continue;
            };
            let Expression::CallExpression(call) = read.expression.get_inner_expression() else {
                continue;
            };
            if member.property.name == "stubEnv"
                && matches!(
                    call.callee.get_inner_expression(),
                    Expression::Identifier(_)
                )
                && call.arguments.is_empty()
                && known_vi(semantic, facts, &member.object)
            {
                out.test_env_calls.insert(call.span.start, stub.span.end);
            }
        }
    }
}

pub(super) fn immediate_read(semantic: &Semantic<'_>, call: NodeId) -> bool {
    let nodes = semantic.nodes();
    let AstKind::CallExpression(expression) = nodes.kind(call) else {
        return false;
    };
    if !matches!(
        expression.callee.get_inner_expression(),
        Expression::Identifier(_)
    ) {
        return false;
    }
    let Some(argument) = expression
        .arguments
        .first()
        .and_then(Argument::as_expression)
    else {
        return false;
    };
    let root = match argument.get_inner_expression() {
        Expression::LogicalExpression(logical)
            if matches!(
                logical.operator,
                LogicalOperator::Coalesce | LogicalOperator::Or
            ) =>
        {
            &logical.left
        }
        other => other,
    };
    if env_name(semantic, root).is_none() {
        return false;
    }
    let returned = nodes.parent_id(call);
    if !matches!(nodes.kind(returned), AstKind::ReturnStatement(_)) {
        return false;
    }
    let body = nodes.parent_id(returned);
    let AstKind::FunctionBody(statements) = nodes.kind(body) else {
        return false;
    };
    let AstKind::Function(function) = nodes.parent_kind(body) else {
        return false;
    };
    statements.statements.len() == 1
        && !function.generator
        && function.params.items.is_empty()
        && function.params.rest.is_none()
}

fn known_vi(
    semantic: &Semantic<'_>,
    facts: &TypeScriptModuleFacts,
    expression: &Expression<'_>,
) -> bool {
    let Expression::Identifier(id) = expression.get_inner_expression() else {
        return false;
    };
    let symbol = id
        .reference_id
        .get()
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id());
    symbol.is_some_and(|symbol| {
        facts.imports.iter().any(|import| {
            import.specifier == "vitest"
                && !import.type_only
                && import.bindings.iter().any(|binding| {
                    !binding.type_only
                        && binding.kind == "named"
                        && binding.imported == "vi"
                        && binding.binding_id == symbol.index()
                })
        })
    })
}
