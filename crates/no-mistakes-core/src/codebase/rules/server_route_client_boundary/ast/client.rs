mod assignment;
mod calls;
mod declarations;
mod modules;
mod patterns;
mod scopes;
mod visit;

use modules::{client_module_expr, is_client_factory_member, is_client_http_module};
use oxc_allocator::Allocator;
use oxc_ast::ast::{BindingPattern, Expression};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_span::SourceType;
use patterns::binding_names;
use scopes::ClientScopes;
use std::path::Path;

const HTTP_VERBS: &[&str] = &[
    "get", "post", "put", "patch", "delete", "del", "head", "options", "all",
];

fn is_client_method_name(name: &str) -> bool {
    HTTP_VERBS.contains(&name) || name == "request"
}

pub(in crate::codebase::rules::server_route_client_boundary) fn client_call_lines(
    path: &Path,
    source: &str,
) -> Vec<usize> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_else(|_| SourceType::ts());
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return Vec::new();
    }
    let mut visitor = ClientHttpVisitor::new(source);
    visitor.visit_program(&parsed.program);
    visitor.lines.sort_unstable();
    visitor.lines.dedup();
    visitor.lines
}

struct ClientHttpVisitor<'a> {
    source: &'a str,
    scopes: ClientScopes,
    in_var_declaration: bool,
    lines: Vec<usize>,
}

impl<'a> ClientHttpVisitor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            scopes: ClientScopes::new(),
            in_var_declaration: false,
            lines: Vec::new(),
        }
    }

    fn enter_scope(&mut self, tracks_var_bindings: bool) {
        self.scopes.enter(tracks_var_bindings);
    }

    fn leave_scope(&mut self) {
        self.scopes.leave();
    }

    fn add_client_name(&mut self, name: String) {
        self.scopes.add_client_name(name, self.in_var_declaration);
    }

    fn add_client_callee_name(&mut self, name: String) {
        self.scopes
            .add_client_callee_name(name, self.in_var_declaration);
    }

    fn add_client_factory_callee_name(&mut self, name: String) {
        self.scopes
            .add_client_factory_callee_name(name, self.in_var_declaration);
    }

    fn shadow_name(&mut self, name: String) {
        self.scopes.shadow_name(name, self.in_var_declaration);
    }

    fn is_shadowed_name(&self, name: &str) -> bool {
        self.scopes.is_shadowed_name(name)
    }

    fn assign_client_name(&mut self, name: String) {
        self.scopes.assign_client_name(name);
    }

    fn assign_client_callee_name(&mut self, name: String) {
        self.scopes.assign_client_callee_name(name);
    }

    fn assign_client_factory_callee_name(&mut self, name: String) {
        self.scopes.assign_client_factory_callee_name(name);
    }

    fn assign_shadow_name(&mut self, name: String) {
        self.scopes.assign_shadow_name(name);
    }

    fn is_client_name(&self, name: &str) -> bool {
        self.scopes.is_client_name(name)
    }

    fn is_client_callee_name(&self, name: &str) -> bool {
        self.scopes.is_client_callee_name(name)
    }

    fn is_client_factory_callee_name(&self, name: &str) -> bool {
        self.scopes.is_client_factory_callee_name(name)
    }

    fn client_expr(&self, expr: &Expression<'_>) -> bool {
        match expr {
            Expression::Identifier(id) => self.is_client_name(id.name.as_str()),
            Expression::ParenthesizedExpression(expr) => self.client_expr(&expr.expression),
            Expression::StaticMemberExpression(member) => {
                if client_module_expr(&member.object) {
                    return modules::client_module_member_expr(member);
                }
                self.client_expr(&member.object)
            }
            Expression::ChainExpression(chain) => chain
                .expression
                .as_member_expression()
                .is_some_and(|member| self.client_expr(member.object())),
            Expression::CallExpression(call) => {
                client_module_expr(expr)
                    || match &call.callee {
                        Expression::Identifier(id) => {
                            self.is_client_factory_callee_name(id.name.as_str())
                        }
                        Expression::StaticMemberExpression(member) => {
                            is_client_factory_member(member.property.name.as_str())
                                && self.client_expr(&member.object)
                        }
                        _ => false,
                    }
            }
            _ => false,
        }
    }

    fn mark_binding_pattern_shadowed(&mut self, pattern: &BindingPattern<'_>) {
        for name in binding_names(pattern) {
            self.shadow_name(name);
        }
    }

    fn mark_parameters_shadowed(&mut self, params: &oxc_ast::ast::FormalParameters<'_>) {
        for param in &params.items {
            self.mark_binding_pattern_shadowed(&param.pattern);
        }
        if let Some(rest) = &params.rest {
            self.mark_binding_pattern_shadowed(&rest.rest.argument);
        }
    }
}
