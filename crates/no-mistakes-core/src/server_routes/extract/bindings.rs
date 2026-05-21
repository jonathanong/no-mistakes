use super::{
    commonjs::{
        commonjs_framework_binding, commonjs_property_is_framework, server_module_from_require,
    },
    helpers::first_object_prefix,
    ServerRouteVisitor,
};
use crate::server_routes::model::Binding;
use crate::server_routes::types::Framework;
use oxc_ast::ast::{Argument, CallExpression, Expression, StaticMemberExpression};

impl ServerRouteVisitor<'_> {
    pub(super) fn binding_from_expr(&self, expr: &Expression<'_>) -> Option<Binding> {
        match expr {
            Expression::CallExpression(call) => self.call_binding(call),
            Expression::NewExpression(new_expr) => {
                let Expression::Identifier(id) = &new_expr.callee else {
                    return None;
                };
                let name = id.name.as_str();
                if self.hono_names.contains(name) {
                    Some(Binding::new(
                        Framework::Hono,
                        first_object_prefix(&new_expr.arguments),
                    ))
                } else if self.koa_router_names.contains(name) {
                    Some(Binding::new(
                        Framework::KoaRouter,
                        first_object_prefix(&new_expr.arguments),
                    ))
                } else {
                    None
                }
            }
            Expression::StaticMemberExpression(member) => self.member_binding(member),
            _ => None,
        }
    }

    pub(super) fn client_http_from_expr(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::Identifier(id) => self.client_http_names.contains(id.name.as_str()),
            Expression::ParenthesizedExpression(expr) => {
                self.client_http_from_expr(&expr.expression)
            }
            Expression::CallExpression(call) => self.client_http_from_call(call),
            Expression::StaticMemberExpression(member) => {
                self.client_http_from_expr(&member.object)
            }
            _ => false,
        }
    }

    pub(super) fn client_http_module_from_expr(&self, expr: &Expression<'_>) -> bool {
        if server_module_from_require(expr).is_some_and(is_client_http_module) {
            return true;
        }
        match expr {
            Expression::ParenthesizedExpression(expr) => {
                self.client_http_module_from_expr(&expr.expression)
            }
            Expression::CallExpression(call) => match &call.callee {
                Expression::StaticMemberExpression(member) => {
                    self.client_http_module_from_expr(&member.object)
                }
                _ => false,
            },
            Expression::StaticMemberExpression(member) => {
                self.client_http_module_from_expr(&member.object)
            }
            _ => false,
        }
    }

    fn client_http_from_call(&self, call: &CallExpression<'_>) -> bool {
        match &call.callee {
            Expression::Identifier(id) => self.client_http_names.contains(id.name.as_str()),
            Expression::StaticMemberExpression(member) => {
                self.client_http_from_expr(&member.object)
            }
            _ => false,
        }
    }

    fn call_binding(&self, call: &CallExpression<'_>) -> Option<Binding> {
        if let Some(binding) = self.inline_commonjs_binding_from_expr(&call.callee) {
            return Some(binding);
        }
        match &call.callee {
            Expression::Identifier(id) if self.express_names.contains(id.name.as_str()) => {
                Some(Binding::new(Framework::Express, None))
            }
            Expression::Identifier(id) if self.api_server_names.contains(id.name.as_str()) => {
                Some(Binding::new(Framework::ApiServer, None))
            }
            Expression::Identifier(id) if self.path_match_names.contains(id.name.as_str()) => {
                Some(Binding::new(Framework::KoaPathMatch, None))
            }
            Expression::StaticMemberExpression(member)
                if member.property.name.as_str() == "Router"
                    && self.express_module_expr(&member.object) =>
            {
                Some(Binding::new(Framework::Express, None))
            }
            Expression::StaticMemberExpression(member)
                if member.property.name.as_str() == "Router" =>
            {
                self.inline_commonjs_binding_from_expr(&member.object)
            }
            Expression::StaticMemberExpression(member)
                if matches!(member.property.name.as_str(), "basePath" | "route") =>
            {
                let mut binding = self.object_binding(&member.object)?;
                if let Some(prefix) = call.arguments.first().and_then(|arg| self.literal_arg(arg)) {
                    binding.prefixes.push(prefix);
                }
                Some(binding)
            }
            _ => None,
        }
    }

    fn inline_commonjs_binding_from_expr(&self, expr: &Expression<'_>) -> Option<Binding> {
        match expr {
            Expression::ParenthesizedExpression(expr) => {
                self.inline_commonjs_binding_from_expr(&expr.expression)
            }
            Expression::CallExpression(call) => self.inline_commonjs_binding(call),
            Expression::StaticMemberExpression(member) => {
                let source = server_module_from_require(&member.object)?;
                commonjs_property_is_framework(source, member.property.name.as_str())
                    .then(|| commonjs_framework_binding(source))
                    .flatten()
            }
            _ => None,
        }
    }

    fn inline_commonjs_binding(&self, call: &CallExpression<'_>) -> Option<Binding> {
        let Expression::Identifier(id) = &call.callee else {
            return None;
        };
        if id.name.as_str() != "require" {
            return None;
        }
        let Some(Argument::StringLiteral(source)) = call.arguments.first() else {
            return None;
        };
        commonjs_framework_binding(source.value.as_str())
    }

    fn member_binding(&self, member: &StaticMemberExpression<'_>) -> Option<Binding> {
        if member.property.name.as_str() == "Router" && self.express_module_expr(&member.object) {
            return Some(Binding::new(Framework::Express, None));
        }
        let source = server_module_from_require(&member.object)?;
        commonjs_property_is_framework(source, member.property.name.as_str())
            .then(|| commonjs_framework_binding(source))
            .flatten()
    }

    fn express_module_expr(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::Identifier(id) => self.express_names.contains(id.name.as_str()),
            Expression::ParenthesizedExpression(expr) => self.express_module_expr(&expr.expression),
            Expression::StaticMemberExpression(member) => {
                member.property.name.as_str() == "default"
                    && self.express_module_expr(&member.object)
            }
            _ => false,
        }
    }

    fn object_binding(&self, object: &Expression<'_>) -> Option<Binding> {
        if let Expression::Identifier(id) = object {
            if let Some(binding) = self.facts.bindings.get(id.name.as_str()) {
                return Some(binding.clone());
            }
        }
        self.binding_from_expr(object)
    }
}

pub(crate) fn is_client_http_module(source: &str) -> bool {
    matches!(
        source,
        "axios"
            | "got"
            | "ky"
            | "supertest"
            | "superagent"
            | "undici"
            | "node-fetch"
            | "http"
            | "https"
            | "node:http"
            | "node:https"
            | "@playwright/test"
    )
}
