use super::ast::is_cache_directive;
use super::patterns::{banned_segment_config, fetch_cache_findings, single_binding_name};
use crate::codebase::rules::nextjs_no_caching::NextjsCachingFinding;
use crate::codebase::ts_source::byte_offset_to_line;
use oxc_ast::ast::{
    Argument, CallExpression, Declaration, ExportDefaultDeclarationKind, Expression, FunctionBody,
    VariableDeclaration,
};
use oxc_ast_visit::{walk, Visit};
use std::collections::{HashMap, HashSet};

pub(super) struct NextjsCachingVisitor<'a> {
    pub(super) source: &'a str,
    pub(super) findings: Vec<NextjsCachingFinding>,
    unstable_cache_bindings: HashSet<String>,
    next_cache_namespaces: HashSet<String>,
    next_config_bindings: HashMap<String, Vec<(u32, String)>>,
}

impl<'a> NextjsCachingVisitor<'a> {
    pub(super) fn new(source: &'a str, findings: Vec<NextjsCachingFinding>) -> Self {
        Self {
            source,
            findings,
            unstable_cache_bindings: HashSet::new(),
            next_cache_namespaces: HashSet::new(),
            next_config_bindings: HashMap::new(),
        }
    }

    fn push(&mut self, byte_offset: u32, message: impl Into<String>) {
        self.findings.push(NextjsCachingFinding {
            line: byte_offset_to_line(self.source, byte_offset as usize) as usize,
            message: message.into(),
        });
    }

    fn check_fetch_call(&mut self, call: &CallExpression<'a>) {
        let Expression::Identifier(callee) = &call.callee else {
            return;
        };
        if callee.name.as_str() != "fetch" {
            return;
        }
        let Some(Argument::ObjectExpression(options)) = call.arguments.get(1) else {
            return;
        };
        for finding in fetch_cache_findings(options) {
            self.push(call.span.start, finding);
        }
    }

    fn check_call(&mut self, call: &CallExpression<'a>) {
        match &call.callee {
            Expression::Identifier(callee)
                if self.unstable_cache_bindings.contains(callee.name.as_str()) =>
            {
                self.push(call.span.start, unstable_cache_message());
            }
            Expression::StaticMemberExpression(member)
                if member.property.name.as_str() == "unstable_cache"
                    && self.is_next_cache_namespace(&member.object) =>
            {
                self.push(call.span.start, unstable_cache_message());
            }
            _ => {}
        }
    }

    fn check_import(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        let Some(effects) = super::cache_imports::effects(import) else {
            return;
        };
        self.unstable_cache_bindings
            .extend(effects.unstable_cache_bindings);
        self.next_cache_namespaces.extend(effects.namespaces);
        for (start, message) in effects.findings {
            self.push(start, message);
        }
    }

    fn check_export(&mut self, export: &oxc_ast::ast::ExportNamedDeclaration<'a>) {
        let Some(Declaration::VariableDeclaration(var_decl)) = export.declaration.as_ref() else {
            return;
        };
        for decl in &var_decl.declarations {
            let Some(name) = single_binding_name(&decl.id) else {
                continue;
            };
            let Some(init) = decl.init.as_ref() else {
                continue;
            };
            if banned_segment_config(name.as_str(), init) {
                self.push(decl.span.start, segment_config_message(&name));
            }
        }
    }

    fn check_default_export(&mut self, export: &oxc_ast::ast::ExportDefaultDeclaration<'a>) {
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(obj) => {
                self.push_next_config_findings(super::config::object_findings(obj));
            }
            ExportDefaultDeclarationKind::CallExpression(call) => {
                self.push_next_config_findings(super::config::call_findings(call));
            }
            ExportDefaultDeclarationKind::Identifier(id) => {
                if let Some(findings) = self.next_config_bindings.get(id.name.as_str()) {
                    self.push_next_config_findings(findings.clone());
                }
            }
            _ => {}
        }
    }

    fn collect_next_config_bindings(&mut self, var: &VariableDeclaration<'a>) {
        for decl in &var.declarations {
            let Some(name) = single_binding_name(&decl.id) else {
                continue;
            };
            let Some(init) = decl.init.as_ref() else {
                continue;
            };
            let findings = super::config::expression_findings(init);
            if !findings.is_empty() {
                self.next_config_bindings.insert(name, findings);
            }
        }
    }

    fn push_next_config_findings(&mut self, findings: Vec<(u32, String)>) {
        for (start, message) in findings {
            self.push(start, message);
        }
    }

    fn check_function_body_directives(&mut self, body: &FunctionBody<'a>) {
        for directive in &body.directives {
            if is_cache_directive(directive.directive.as_str()) {
                self.push(
                    directive.span.start,
                    "Next.js cache directives are disabled; remove this directive",
                );
            }
        }
    }

    fn is_next_cache_namespace(&self, expr: &Expression<'a>) -> bool {
        matches!(
            expr,
            Expression::Identifier(id) if self.next_cache_namespaces.contains(id.name.as_str())
        )
    }
}

impl<'a> Visit<'a> for NextjsCachingVisitor<'a> {
    fn visit_import_declaration(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        self.check_import(import);
        walk::walk_import_declaration(self, import);
    }

    fn visit_export_named_declaration(
        &mut self,
        export: &oxc_ast::ast::ExportNamedDeclaration<'a>,
    ) {
        self.check_export(export);
        walk::walk_export_named_declaration(self, export);
    }

    fn visit_variable_declaration(&mut self, var: &VariableDeclaration<'a>) {
        self.collect_next_config_bindings(var);
        walk::walk_variable_declaration(self, var);
    }

    fn visit_export_default_declaration(
        &mut self,
        export: &oxc_ast::ast::ExportDefaultDeclaration<'a>,
    ) {
        self.check_default_export(export);
        walk::walk_export_default_declaration(self, export);
    }

    fn visit_function_body(&mut self, body: &FunctionBody<'a>) {
        self.check_function_body_directives(body);
        walk::walk_function_body(self, body);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.check_call(call);
        self.check_fetch_call(call);
        walk::walk_call_expression(self, call);
    }
}

fn unstable_cache_message() -> &'static str {
    "Next.js unstable_cache is disabled; compute this value per request"
}

fn segment_config_message(name: &str) -> String {
    format!("Next.js `{name}` cache segment config is disabled; remove static caching")
}
