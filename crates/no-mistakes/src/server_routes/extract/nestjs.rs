use super::{helpers::method_name, ServerRouteVisitor};
use crate::server_routes::model::{Binding, RouteSite};
use crate::server_routes::normalize::join_paths;
use crate::server_routes::source::line_number;
use crate::server_routes::types::Framework;
use oxc_ast::ast::{
    Argument, Class, ClassElement, Decorator, Expression, MethodDefinition, ObjectPropertyKind,
};

const NESTJS_COMMON: &str = "@nestjs/common";

impl ServerRouteVisitor<'_> {
    pub(super) fn record_nestjs_class(&mut self, class: &Class<'_>) {
        let Some(prefix) = self.nestjs_controller_prefix(&class.decorators) else {
            return;
        };
        let binding = class
            .id
            .as_ref()
            .map(|id| id.name.to_string())
            .unwrap_or_else(|| "default".to_string());
        self.facts.bindings.insert(
            binding.clone(),
            Binding::new(
                Framework::Nestjs,
                Some(prefix.clone()).filter(|path| !path.is_empty()),
            ),
        );
        for element in &class.body.body {
            let ClassElement::MethodDefinition(method) = element else {
                continue;
            };
            if method.r#static {
                continue;
            }
            self.record_nestjs_method(&binding, method);
        }
    }

    fn record_nestjs_method(&mut self, binding: &str, method: &MethodDefinition<'_>) {
        for decorator in &method.decorators {
            let Some((http_method, path, start)) = self.nestjs_verb_decorator(decorator) else {
                continue;
            };
            let raw_path = join_paths("", &path);
            self.facts.routes.push(RouteSite {
                file: self.path.to_path_buf(),
                line: line_number(self.source, start),
                binding: binding.to_string(),
                method: method_name(&http_method),
                raw_path: raw_path.clone(),
                path: raw_path,
                query_params: Vec::new(),
                framework: Framework::Nestjs,
            });
        }
    }

    fn nestjs_controller_prefix(&self, decorators: &[Decorator<'_>]) -> Option<String> {
        decorators.iter().find_map(|decorator| {
            let (name, _) = decorator_callee(decorator)?;
            if !self.nestjs_imported(name, "Controller") {
                return None;
            }
            self.decorator_path(decorator)
        })
    }

    fn nestjs_verb_decorator(&self, decorator: &Decorator<'_>) -> Option<(String, String, u32)> {
        let (name, start) = decorator_callee(decorator)?;
        let imported = self.nestjs_imported_name(name)?;
        let method = nestjs_http_method(imported)?;
        let path = self.decorator_path(decorator)?;
        Some((method, path, start))
    }

    fn decorator_path(&self, decorator: &Decorator<'_>) -> Option<String> {
        let Expression::CallExpression(call) = &decorator.expression else {
            return Some(String::new());
        };
        let Some(arg) = call.arguments.first() else {
            return Some(String::new());
        };
        if let Argument::ObjectExpression(object) = arg {
            for prop in &object.properties {
                let ObjectPropertyKind::ObjectProperty(property) = prop else {
                    continue;
                };
                if property.key.static_name().as_deref() != Some("path") {
                    continue;
                }
                return super::const_string(&property.value);
            }
            return Some(String::new());
        }
        arg.as_expression().and_then(super::const_string)
    }

    fn nestjs_imported(&self, local: &str, imported: &str) -> bool {
        self.nestjs_imported_name(local) == Some(imported)
    }

    fn nestjs_imported_name<'a>(&'a self, local: &str) -> Option<&'a str> {
        self.facts.imports.iter().find_map(|import| {
            (import.source == NESTJS_COMMON && import.local == local)
                .then_some(import.imported.as_str())
        })
    }
}

fn decorator_callee<'a>(decorator: &'a Decorator<'a>) -> Option<(&'a str, u32)> {
    match &decorator.expression {
        Expression::Identifier(id) => Some((id.name.as_str(), decorator.span.start)),
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(id) => Some((id.name.as_str(), call.span.start)),
            _ => None,
        },
        _ => None,
    }
}

fn nestjs_http_method(imported: &str) -> Option<String> {
    match imported {
        "Get" | "Post" | "Put" | "Patch" | "Delete" | "Head" | "Options" | "All" => {
            Some(imported.to_ascii_lowercase())
        }
        _ => None,
    }
}
