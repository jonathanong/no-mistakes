use crate::server_routes::model::Binding;
use crate::server_routes::types::Framework;
use oxc_ast::ast::{Argument, Expression};

pub(super) fn server_module_from_require<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::ParenthesizedExpression(expr) => server_module_from_require(&expr.expression),
        Expression::StaticMemberExpression(member) => {
            let source = server_module_from_require(&member.object)?;
            let property = member.property.name.as_str();
            (property == "default" || commonjs_property_is_framework(source, property))
                .then_some(source)
        }
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(id) if id.name.as_str() == "require" => {
                match call.arguments.first() {
                    Some(Argument::StringLiteral(value)) => Some(value.value.as_str()),
                    _ => None,
                }
            }
            Expression::Identifier(id) if is_commonjs_interop_wrapper(id.name.as_str()) => call
                .arguments
                .first()
                .and_then(Argument::as_expression)
                .and_then(server_module_from_require),
            _ => None,
        },
        _ => None,
    }
}

fn is_commonjs_interop_wrapper(name: &str) -> bool {
    matches!(
        name,
        "__importDefault" | "__importStar" | "_interopRequireDefault" | "_interopRequireWildcard"
    )
}

pub(super) fn commonjs_framework_binding(source: &str) -> Option<Binding> {
    let framework = match source {
        "express" => Framework::Express,
        "hono" | "@hono/hono" => Framework::Hono,
        "@koa/router" | "koa-router" => Framework::KoaRouter,
        "koa-path-match" | "@koa/path-match" => Framework::KoaPathMatch,
        "@jongleberry/api-server" | "api-server" => Framework::ApiServer,
        _ => return None,
    };
    Some(Binding::new(framework, None))
}

pub(super) fn commonjs_property_is_framework(source: &str, key: &str) -> bool {
    const FRAMEWORK_PROPERTIES: &[(&str, &str)] = &[
        ("express", "Router"),
        ("hono", "Hono"),
        ("@hono/hono", "Hono"),
        ("@koa/router", "Router"),
        ("koa-router", "Router"),
        ("koa-path-match", "default"),
        ("@koa/path-match", "default"),
        ("@jongleberry/api-server", "createApp"),
        ("api-server", "createApp"),
    ];
    FRAMEWORK_PROPERTIES.contains(&(source, key))
}
