//! Runtime package imports in a published entry module.

use super::specifier::package_name;
use crate::ast::{js_literal, template_quasi_text};
use crate::codebase::ts_source::byte_offset_to_line;
use crate::fetch::import_shape::{is_runtime_export, is_runtime_import};
use oxc_ast::ast::{
    Argument, ExportAllDeclaration, ExportFromDeclaration, Expression, ImportDeclaration,
    ImportExpression, ImportOrExportKind,
};
use oxc_ast_visit::{walk, Visit};

pub(super) struct RuntimeImport {
    pub package: String,
    pub line: usize,
}

pub(super) fn collect(path: &std::path::Path, source: &str) -> anyhow::Result<Vec<RuntimeImport>> {
    crate::ast::with_program(path, source, |program, source| {
        let mut visitor = ImportVisitor {
            source,
            imports: Vec::new(),
        };
        visitor.visit_program(program);
        visitor.imports
    })
}

struct ImportVisitor<'a> {
    source: &'a str,
    imports: Vec<RuntimeImport>,
}

impl ImportVisitor<'_> {
    fn record(&mut self, specifier: &str, start: u32) {
        let Some(package) = package_name(specifier) else {
            return;
        };
        self.imports.push(RuntimeImport {
            package,
            line: byte_offset_to_line(self.source, start as usize) as usize,
        });
    }
}

impl<'a> Visit<'a> for ImportVisitor<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        if is_runtime_import(decl) {
            self.record(&js_literal(&decl.source), decl.source.span.start);
        }
    }

    fn visit_export_all_declaration(&mut self, decl: &ExportAllDeclaration<'a>) {
        if decl.export_kind != ImportOrExportKind::Type {
            self.record(&js_literal(&decl.source), decl.source.span.start);
        }
    }

    fn visit_export_from_declaration(&mut self, decl: &ExportFromDeclaration<'a>) {
        if is_runtime_export(decl) {
            self.record(&js_literal(&decl.source), decl.source.span.start);
        }
        walk::walk_export_from_declaration(self, decl);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        if let Some(text) = static_string(&expr.source) {
            self.record(&text, expr.span.start);
        }
    }

    fn visit_call_expression(&mut self, expr: &oxc_ast::ast::CallExpression<'a>) {
        if let Expression::Identifier(callee) = &expr.callee {
            if callee.name == "require" {
                if let Some(text) = expr.arguments.first().and_then(argument_string) {
                    self.record(&text, expr.span.start);
                }
            }
        }
        walk::walk_call_expression(self, expr);
    }
}

fn static_string(expr: &Expression<'_>) -> Option<String> {
    match expr {
        Expression::StringLiteral(literal) => Some(js_literal(literal).into_owned()),
        Expression::TemplateLiteral(template)
            if template.expressions.is_empty() && template.quasis.len() == 1 =>
        {
            cooked(template)
        }
        Expression::ParenthesizedExpression(inner) => static_string(&inner.expression),
        _ => None,
    }
}

fn argument_string(argument: &Argument<'_>) -> Option<String> {
    match argument {
        Argument::StringLiteral(literal) => Some(js_literal(literal).into_owned()),
        Argument::TemplateLiteral(template)
            if template.expressions.is_empty() && template.quasis.len() == 1 =>
        {
            cooked(template)
        }
        Argument::ParenthesizedExpression(inner) => static_string(&inner.expression),
        _ => None,
    }
}

fn cooked(template: &oxc_ast::ast::TemplateLiteral<'_>) -> Option<String> {
    template
        .quasis
        .first()
        .map(|quasi| template_quasi_text(quasi).into_owned())
}
