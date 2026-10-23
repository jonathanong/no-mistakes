//! Literal-only caching projection. No const scope, alias, or wrapper evaluation.
use super::{ast, bindings::TopLevelBindings, NextjsCachingFinding, NextjsCachingVisitor};
use oxc_ast::ast::*;
use std::{collections::HashMap, path::Path};

pub(crate) struct Collector<'a> {
    base: NextjsCachingVisitor<'a>,
    segment: bool,
    config: bool,
    local_fetch: bool,
}

impl<'a> Collector<'a> {
    pub(crate) fn new(path: &Path, source: &'a str, program: &Program<'_>) -> Self {
        // This cheap syntactic scan preserves max's file-level fetch ownership
        // without constructing any static-value or config-binding tables.
        let local_fetch = program.body.iter().any(|statement| match statement {
            Statement::ImportDeclaration(import) => import.specifiers.as_ref().is_some_and(|specs| specs.iter().any(|spec| spec.local().name == "fetch")),
            Statement::VariableDeclaration(var) => declares_fetch(var),
            Statement::ExportDeclaration(export) => matches!(&export.declaration, Declaration::VariableDeclaration(var) if declares_fetch(var)),
            _ => false,
        });
        let mut base = NextjsCachingVisitor::new(
            source,
            Vec::new(),
            TopLevelBindings {
                next_config: HashMap::new(),
                segment_config: HashMap::new(),
                local_fetch,
                static_consts: Vec::new(),
            },
            false,
            false,
        );
        for directive in &program.directives {
            if ast::is_cache_directive(directive.directive.as_str()) {
                base.push(
                    directive.span.start,
                    "Next.js cache directives are disabled; remove this directive",
                );
            }
        }
        Self {
            base,
            segment: ast::is_route_segment_file(path),
            config: ast::is_next_config_file(path),
            local_fetch,
        }
    }

    pub(crate) fn finish(self) -> Vec<NextjsCachingFinding> {
        super::finish_visitor(self.base)
    }
    fn unknown(&mut self, offset: u32) {
        self.base.push(offset, "Next.js caching configuration is unanalyzable at effort low; write literal fetch options, literal route segment exports, or a literal next.config object directly at the use site");
        self.base.findings.last_mut().unwrap().unanalyzable = true;
    }
    pub(crate) fn import_declaration(&mut self, node: &ImportDeclaration<'a>) {
        self.base.check_import(node);
    }
    pub(crate) fn function_body(&mut self, node: &FunctionBody<'a>) {
        self.base.check_function_body_directives(node);
    }
    pub(crate) fn call_expression(&mut self, call: &CallExpression<'a>) {
        self.base.check_call(call);
        if !matches!(&call.callee, Expression::Identifier(id) if id.name == "fetch")
            || self.local_fetch
        {
            return;
        }
        let Some(argument) = call.arguments.get(1) else {
            return;
        };
        let Argument::ObjectExpression(object) = argument else {
            self.unknown(call.span.start);
            return;
        };
        for message in super::patterns::fetch_cache_findings(object, &[]) {
            self.base.push(call.span.start, message);
        }
        self.object(object, Context::Fetch);
    }
    pub(crate) fn export_declaration(&mut self, export: &ExportDeclaration<'a>) {
        if !self.segment {
            return;
        }
        let Declaration::VariableDeclaration(var) = &export.declaration else {
            return;
        };
        for declaration in &var.declarations {
            let Some(name) = super::patterns::single_binding_name(&declaration.id) else {
                continue;
            };
            if !segment_name(&name) {
                continue;
            }
            if let Some(value) = &declaration.init {
                if literal(value) {
                    if super::patterns::banned_segment_config(&name, value, &[]) {
                        self.base.push(
                            declaration.span.start,
                            super::visitor::segment_config_message(&name),
                        );
                    }
                } else {
                    self.unknown(declaration.span.start);
                }
            } else {
                self.unknown(declaration.span.start);
            }
        }
    }
    pub(crate) fn export_named_declaration(&mut self, export: &ExportNamedDeclaration<'a>) {
        if self.segment {
            for specifier in &export.specifiers {
                if segment_name(specifier.exported.name().as_str()) {
                    self.unknown(specifier.span.start);
                }
            }
        }
    }
    pub(crate) fn export_default_declaration(&mut self, export: &ExportDefaultDeclaration<'a>) {
        if !self.config {
            return;
        }
        if let ExportDefaultDeclarationKind::ObjectExpression(object) = &export.declaration {
            self.config_object(object);
        } else {
            self.unknown(export.span.start);
        }
    }
    pub(crate) fn assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if !self.config {
            return;
        }
        if !matches!(&assignment.left, AssignmentTarget::StaticMemberExpression(member) if member.property.name == "exports" && matches!(&member.object, Expression::Identifier(id) if id.name == "module"))
        {
            return;
        }
        if let Expression::ObjectExpression(object) = &assignment.right {
            self.config_object(object);
        } else {
            self.unknown(assignment.span.start);
        }
    }
    fn config_object(&mut self, object: &ObjectExpression<'a>) {
        for (offset, message) in super::config::object_findings(object, &[]) {
            self.base.push(offset, message);
        }
        self.object(object, Context::Config);
    }
    fn object(&mut self, object: &ObjectExpression<'a>, context: Context) {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                self.unknown(object.span.start);
                continue;
            };
            let Some(name) = super::const_values::property_name(&property.key) else {
                self.unknown(property.span.start);
                continue;
            };
            match (context, name.as_str()) {
                (Context::Fetch, "next") | (Context::Config, "experimental") => {
                    if let Expression::ObjectExpression(nested) = &property.value {
                        self.object(
                            nested,
                            if matches!(context, Context::Fetch) {
                                Context::Next
                            } else {
                                Context::Config
                            },
                        );
                    } else {
                        self.unknown(property.span.start);
                    }
                }
                (Context::Fetch, "cache")
                | (Context::Next, "revalidate")
                | (Context::Config, "cacheComponents") => {
                    if !literal(&property.value) {
                        self.unknown(property.span.start);
                    }
                }
                (Context::Next, "tags") => {
                    if !matches!(property.value, Expression::ArrayExpression(_)) {
                        self.unknown(property.span.start);
                    }
                }
                _ => {}
            }
        }
    }
}
#[derive(Clone, Copy)]
enum Context {
    Fetch,
    Next,
    Config,
}
fn literal(value: &Expression<'_>) -> bool {
    match crate::codebase::ts_source::unwrap_ts_wrappers(value) {
        Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::NullLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}

fn declares_fetch(var: &VariableDeclaration<'_>) -> bool {
    var.declarations.iter().any(|declaration| matches!(&declaration.id, BindingPattern::BindingIdentifier(id) if id.name == "fetch"))
}
fn segment_name(name: &str) -> bool {
    matches!(name, "revalidate" | "fetchCache" | "dynamic")
}

pub(super) fn extract_program(
    path: &Path,
    source: &str,
    program: &Program<'_>,
) -> Vec<NextjsCachingFinding> {
    use oxc_ast_visit::Visit;
    let mut collector = Collector::new(path, source, program);
    collector.visit_program(program);
    collector.finish()
}
impl<'a> oxc_ast_visit::Visit<'a> for Collector<'a> {
    fn visit_import_declaration(&mut self, node: &ImportDeclaration<'a>) {
        self.import_declaration(node);
        oxc_ast_visit::walk::walk_import_declaration(self, node);
    }
    fn visit_call_expression(&mut self, node: &CallExpression<'a>) {
        self.call_expression(node);
        oxc_ast_visit::walk::walk_call_expression(self, node);
    }
    fn visit_export_declaration(&mut self, node: &ExportDeclaration<'a>) {
        self.export_declaration(node);
        oxc_ast_visit::walk::walk_export_declaration(self, node);
    }
    fn visit_export_named_declaration(&mut self, node: &ExportNamedDeclaration<'a>) {
        self.export_named_declaration(node);
        oxc_ast_visit::walk::walk_export_named_declaration(self, node);
    }
    fn visit_export_default_declaration(&mut self, node: &ExportDefaultDeclaration<'a>) {
        self.export_default_declaration(node);
        oxc_ast_visit::walk::walk_export_default_declaration(self, node);
    }
    fn visit_assignment_expression(&mut self, node: &AssignmentExpression<'a>) {
        self.assignment_expression(node);
        oxc_ast_visit::walk::walk_assignment_expression(self, node);
    }
    fn visit_function_body(&mut self, node: &FunctionBody<'a>) {
        self.function_body(node);
        oxc_ast_visit::walk::walk_function_body(self, node);
    }
}
