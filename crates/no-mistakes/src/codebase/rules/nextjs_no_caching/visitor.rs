use super::ast::is_cache_directive;
use super::patterns::{
    banned_next_cache_import, banned_segment_config, boolean_value, fetch_cache_findings,
    single_binding_name,
};
use crate::codebase::rules::nextjs_no_caching::NextjsCachingFinding;
use crate::codebase::ts_source::{byte_offset_to_line, static_property_key_name};
use oxc_ast::ast::{
    Argument, CallExpression, Declaration, ExportDefaultDeclarationKind, Expression, FunctionBody,
    ImportDeclarationSpecifier, ObjectExpression, ObjectPropertyKind,
};
use oxc_ast_visit::{walk, Visit};

pub(super) struct NextjsCachingVisitor<'a> {
    pub(super) source: &'a str,
    pub(super) findings: Vec<NextjsCachingFinding>,
}

impl<'a> NextjsCachingVisitor<'a> {
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
            Expression::Identifier(callee) if callee.name.as_str() == "unstable_cache" => {
                self.push(call.span.start, unstable_cache_message());
            }
            Expression::StaticMemberExpression(member)
                if member.property.name.as_str() == "unstable_cache" =>
            {
                self.push(call.span.start, unstable_cache_message());
            }
            _ => {}
        }
    }

    fn check_import(&mut self, import: &oxc_ast::ast::ImportDeclaration<'a>) {
        if import.source.value.as_str() != "next/cache" {
            return;
        }
        let Some(specifiers) = import.specifiers.as_ref() else {
            return;
        };
        for specifier in specifiers {
            self.check_import_specifier(specifier);
        }
    }

    fn check_import_specifier(&mut self, specifier: &ImportDeclarationSpecifier<'a>) {
        match specifier {
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => self.push(
                spec.span.start,
                "next/cache namespace imports are disabled; avoid Next.js cache APIs",
            ),
            ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                let imported = spec.imported.name();
                if banned_next_cache_import(&imported) {
                    self.push(
                        spec.span.start,
                        format!("next/cache `{imported}` is disabled; avoid Next.js cache APIs"),
                    );
                }
            }
            ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => self.push(
                spec.span.start,
                "next/cache default imports are disabled; avoid Next.js cache APIs",
            ),
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
        if let ExportDefaultDeclarationKind::ObjectExpression(obj) = &export.declaration {
            self.check_next_config_object(obj);
        }
    }

    fn check_next_config_object(&mut self, obj: &ObjectExpression<'a>) {
        for prop in &obj.properties {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                continue;
            };
            let Some(name) = static_property_key_name(&prop.key) else {
                continue;
            };
            match name {
                "cacheComponents" if boolean_value(&prop.value) == Some(true) => self.push(
                    prop.span.start,
                    "Next.js cacheComponents config is disabled; remove static caching",
                ),
                "cacheLife" | "cacheHandlers" => {
                    self.push(prop.span.start, next_config_message(name));
                }
                _ => {}
            }
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

fn next_config_message(name: &str) -> String {
    format!("Next.js `{name}` config is disabled; remove static caching")
}
