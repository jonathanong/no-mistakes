mod bindings;
mod commonjs;
mod helpers;
mod imports;
mod literals;
mod named_handlers;
mod nestjs;
mod query_params;
mod records;
mod shape;

use crate::server_routes::model::FileFacts;
use oxc_ast::ast::{
    CallExpression, ExportDefaultDeclarationKind, Expression, ImportOrExportKind, ModuleExportName,
    TSImportEqualsDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::Path;

pub(crate) use commonjs::is_client_http_module;
pub(crate) use shape::has_server_route_shape_from_program;

pub(super) const VERBS: &[&str] = &[
    "get", "post", "put", "patch", "delete", "del", "head", "options", "all",
];

pub(crate) fn extract_program(
    path: &Path,
    source: &str,
    program: &oxc_ast::ast::Program<'_>,
) -> FileFacts {
    let mut visitor = ServerRouteVisitor::new(path, source);
    visitor.pre_collect_named_handlers(program);
    visitor.visit_program(program);
    visitor.facts
}

pub(super) struct ServerRouteVisitor<'a> {
    pub(super) path: &'a Path,
    pub(super) source: &'a str,
    pub(super) facts: FileFacts,
    pub(super) const_strings: HashMap<String, String>,
    pub(super) express_names: HashSet<String>,
    pub(super) fastify_names: HashSet<String>,
    pub(super) hono_names: HashSet<String>,
    pub(super) koa_router_names: HashSet<String>,
    pub(super) path_match_names: HashSet<String>,
    pub(super) api_server_names: HashSet<String>,
    pub(super) client_http_names: HashSet<String>,
    pub(super) named_handler_query_params: HashMap<String, BTreeSet<String>>,
}

impl<'a> Visit<'a> for ServerRouteVisitor<'a> {
    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        let source = import.source.value.as_str().to_string();
        if let Some(specifiers) = &import.specifiers {
            for specifier in specifiers {
                self.record_import(&source, specifier);
            }
        }
        walk::walk_import_declaration(self, import);
    }

    fn visit_variable_declarator(&mut self, decl: &oxc_ast::ast::VariableDeclarator<'a>) {
        let init = decl.init.as_ref();
        let commonjs_source = init.and_then(|init| commonjs::server_module_from_require(init));
        if let Some(source) = commonjs_source {
            self.record_commonjs_pattern(&decl.id, source);
        }
        if let Some(init) = init {
            self.record_destructured_bindings(&decl.id, init);
        }
        let Some(name) = helpers::binding_name(&decl.id) else {
            walk::walk_variable_declarator(self, decl);
            return;
        };
        if let Some(init) = init {
            if let Some(value) = const_string(init) {
                self.const_strings.insert(name.clone(), value);
            }
            if self.client_http_module_from_expr(init) || self.client_http_from_expr(init) {
                self.client_http_names.insert(name.clone());
            }
            if let Some(source) = commonjs_source {
                self.record_commonjs_module(&name, source);
            }
            if let Some(binding) = self.binding_from_expr(init) {
                self.facts.bindings.insert(name, binding);
            }
        }
        walk::walk_variable_declarator(self, decl);
    }

    fn visit_ts_import_equals_declaration(&mut self, import: &TSImportEqualsDeclaration<'a>) {
        if import.import_kind == ImportOrExportKind::Value {
            self.record_ts_import_equals(import);
        }
        walk::walk_ts_import_equals_declaration(self, import);
    }

    fn visit_export_named_declaration(
        &mut self,
        export: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        for specifier in &export.specifiers {
            let exported = module_export_name(&specifier.exported);
            let local = module_export_name(&specifier.local);
            self.facts.exports.insert(exported, local);
        }
        walk::walk_export_named_declaration(self, export);
    }

    fn visit_export_declaration(&mut self, export: &oxc_ast::ast::ExportDeclaration<'a>) {
        if let oxc_ast::ast::Declaration::VariableDeclaration(var_decl) = &export.declaration {
            for decl in &var_decl.declarations {
                if let Some(name) = helpers::binding_name(&decl.id) {
                    self.facts.exports.insert(name.clone(), name);
                }
            }
        }
        walk::walk_export_declaration(self, export);
    }

    fn visit_export_default_declaration(
        &mut self,
        export: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        let local =
            default_export_name(&export.declaration).unwrap_or_else(|| "default".to_string());
        self.facts.exports.insert("default".to_string(), local);
        walk::walk_export_default_declaration(self, export);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.record_call(call);
        walk::walk_call_expression(self, call);
    }

    fn visit_class(&mut self, class: &oxc_ast::ast::Class<'a>) {
        self.record_nestjs_class(class);
        walk::walk_class(self, class);
    }
}

pub(super) fn module_export_name(name: &ModuleExportName<'_>) -> String {
    match name {
        ModuleExportName::IdentifierName(id) => id.name.to_string(),
        ModuleExportName::IdentifierReference(id) => id.name.to_string(),
        ModuleExportName::StringLiteral(value) => value.value.to_string(),
    }
}

fn default_export_name(decl: &ExportDefaultDeclarationKind<'_>) -> Option<String> {
    match decl {
        ExportDefaultDeclarationKind::Identifier(id) => Some(id.name.to_string()),
        _ => None,
    }
}

pub(super) fn const_string(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::StringLiteral(value) => Some(value.value.as_str().to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => Some(
            template
                .quasis
                .iter()
                .filter_map(|quasi| quasi.value.cooked.as_deref())
                .collect::<Vec<_>>()
                .join(""),
        ),
        _ => None,
    }
}

impl<'a> ServerRouteVisitor<'a> {
    fn new(path: &'a Path, source: &'a str) -> Self {
        Self {
            path,
            source,
            facts: FileFacts::default(),
            const_strings: HashMap::new(),
            express_names: HashSet::new(),
            fastify_names: HashSet::new(),
            hono_names: HashSet::new(),
            koa_router_names: HashSet::new(),
            path_match_names: HashSet::new(),
            api_server_names: HashSet::new(),
            client_http_names: HashSet::new(),
            named_handler_query_params: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests;
