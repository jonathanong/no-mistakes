use super::{
    bindings::is_client_http_module, commonjs::commonjs_property_is_framework, const_string,
    helpers::binding_names, import_names, ServerRouteVisitor,
};
use crate::server_routes::model::ImportBinding;
use oxc_ast::ast::{
    BindingPattern, Expression, ImportDeclarationSpecifier, TSImportEqualsDeclaration,
    TSModuleReference,
};

impl ServerRouteVisitor<'_> {
    pub(super) fn record_import(
        &mut self,
        source: &str,
        specifier: &ImportDeclarationSpecifier<'_>,
    ) {
        let (local, imported) = import_names(specifier);
        match source {
            "express" if imported == "default" || imported == "Router" || imported == "*" => {
                self.express_names.insert(local.clone());
            }
            "hono" | "@hono/hono" if imported == "Hono" => {
                self.hono_names.insert(local.clone());
            }
            "@koa/router" | "koa-router" if imported == "default" || imported == "Router" => {
                self.koa_router_names.insert(local.clone());
            }
            "koa-path-match" | "@koa/path-match" if imported == "default" => {
                self.path_match_names.insert(local.clone());
            }
            "@jongleberry/api-server" | "api-server" if imported == "createApp" => {
                self.api_server_names.insert(local.clone());
            }
            _ => {}
        }
        if is_client_http_module(source) {
            self.client_http_names.insert(local.clone());
        }
        self.facts.imports.push(ImportBinding {
            local,
            imported,
            source: source.to_string(),
        });
    }

    pub(super) fn record_commonjs_module(&mut self, local: &str, source: &str) {
        match source {
            "express" => {
                self.express_names.insert(local.to_string());
            }
            "hono" | "@hono/hono" => {
                self.hono_names.insert(local.to_string());
            }
            "@koa/router" | "koa-router" => {
                self.koa_router_names.insert(local.to_string());
            }
            "koa-path-match" | "@koa/path-match" => {
                self.path_match_names.insert(local.to_string());
            }
            "@jongleberry/api-server" | "api-server" => {
                self.api_server_names.insert(local.to_string());
            }
            _ => {}
        }
    }

    pub(super) fn record_commonjs_pattern(&mut self, pattern: &BindingPattern<'_>, source: &str) {
        let BindingPattern::ObjectPattern(object) = pattern else {
            return;
        };
        for prop in &object.properties {
            let Some(key) = prop.key.static_name() else {
                continue;
            };
            for local in binding_names(&prop.value) {
                if commonjs_property_is_framework(source, key.as_ref()) {
                    self.record_commonjs_module(&local, source);
                }
                if is_client_http_module(source) {
                    self.client_http_names.insert(local);
                }
            }
        }
    }

    pub(super) fn record_destructured_bindings(
        &mut self,
        pattern: &BindingPattern<'_>,
        init: &Expression<'_>,
    ) {
        let BindingPattern::ObjectPattern(_) = pattern else {
            return;
        };
        let const_value = const_string(init);
        let is_client = self.client_http_module_from_expr(init) || self.client_http_from_expr(init);
        let binding = self.binding_from_expr(init);
        for name in binding_names(pattern) {
            if let Some(value) = &const_value {
                self.const_strings.insert(name.clone(), value.clone());
            }
            if is_client {
                self.client_http_names.insert(name.clone());
            }
            if let Some(binding) = &binding {
                self.facts.bindings.insert(name, binding.clone());
            }
        }
    }

    pub(super) fn record_ts_import_equals(&mut self, import: &TSImportEqualsDeclaration<'_>) {
        let Some(source) = ts_external_module_reference(&import.module_reference) else {
            return;
        };
        let local = import.id.name.as_str();
        self.record_commonjs_module(local, source);
        if is_client_http_module(source) {
            self.client_http_names.insert(local.to_string());
        }
        self.facts.imports.push(ImportBinding {
            local: local.to_string(),
            imported: "default".to_string(),
            source: source.to_string(),
        });
    }
}

fn ts_external_module_reference<'a>(reference: &'a TSModuleReference<'a>) -> Option<&'a str> {
    match reference {
        TSModuleReference::ExternalModuleReference(reference) => {
            Some(reference.expression.value.as_str())
        }
        _ => None,
    }
}
