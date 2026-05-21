use oxc_ast::ast::{Argument, Expression};

pub(super) fn client_module_expr(expr: &Expression<'_>) -> bool {
    client_module_source(expr).is_some()
}

fn client_module_source<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::ParenthesizedExpression(expr) => client_module_source(&expr.expression),
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(id) if id.name.as_str() == "require" => call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(|expr| match expr {
                    Expression::StringLiteral(source) => Some(source.value.as_str()),
                    _ => None,
                })
                .filter(|source| is_client_http_module(source)),
            Expression::Identifier(id) if is_commonjs_interop_wrapper(id.name.as_str()) => call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(client_module_source),
            Expression::StaticMemberExpression(member) => {
                client_module_member_expr(member).then(|| client_module_source(&member.object))?
            }
            _ => None,
        },
        Expression::StaticMemberExpression(member) => {
            client_module_member_expr(member).then(|| client_module_source(&member.object))?
        }
        _ => None,
    }
}

pub(super) fn client_module_member_expr(member: &oxc_ast::ast::StaticMemberExpression<'_>) -> bool {
    matches!(
        member.property.name.as_str(),
        "create" | "default" | "extend"
    ) && client_module_expr(&member.object)
}

pub(super) fn is_client_factory_member(name: &str) -> bool {
    matches!(name, "create" | "extend")
}

fn is_commonjs_interop_wrapper(name: &str) -> bool {
    matches!(
        name,
        "__importDefault" | "__importStar" | "_interopRequireDefault" | "_interopRequireWildcard"
    )
}

pub(super) fn is_client_named_binding(source: &str, name: &str) -> bool {
    match source {
        "@playwright/test" => name == "request",
        _ => is_client_http_module(source) && name == "default",
    }
}

pub(super) fn is_client_named_callee(source: &str, name: &str) -> bool {
    match source {
        "undici" => [
            "connect", "fetch", "pipeline", "request", "stream", "upgrade",
        ]
        .contains(&name),
        "http" | "https" | "node:http" | "node:https" => name == "request",
        "axios" | "got" | "ky" | "superagent" | "supertest" => name == "request",
        _ => false,
    }
}

pub(super) fn is_client_http_module(source: &str) -> bool {
    crate::server_routes::is_client_http_module(source)
}
