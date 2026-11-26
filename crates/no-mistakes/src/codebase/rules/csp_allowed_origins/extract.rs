use super::{CollectedOrigins, CspDirective};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    ArrayExpression, ArrowFunctionExpression, BinaryExpression, BindingPattern, CallExpression,
    ConditionalExpression, Expression, Function, JSXOpeningElement, LogicalExpression,
    NewExpression, ObjectExpression, Program, ReturnStatement, StringLiteral, TSType,
    TemplateLiteral, VariableDeclarator,
};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;
use std::collections::HashSet;

mod csp_scope;
mod csp_strings;
mod jsx;
mod url;

use csp_strings::{
    is_csp_identifier, is_static_csp_policy_string, try_collect_array, try_collect_binary,
    try_collect_call, try_collect_conditional, try_collect_logical, try_collect_object,
    try_collect_template,
};
use jsx::record_jsx_element;
use url::is_global_or_bare;

pub(crate) fn collect_for_plan(
    path: &std::path::Path,
    source: &str,
    program: &Program<'_>,
    plan: &crate::codebase::check_facts::CheckFactPlan,
) -> CollectedOrigins {
    if plan.csp_allowed_origins_paths.contains(path) {
        collect(program, source)
    } else {
        CollectedOrigins::default()
    }
}

fn collect(program: &Program<'_>, source: &str) -> CollectedOrigins {
    let mut visitor = OriginVisitor {
        source,
        collected: CollectedOrigins::default(),
        csp_depth: 0,
        in_csp_fn: false,
        resolved_bindings: HashSet::new(),
    };
    visitor.visit_program(program);
    visitor.collected
}

struct OriginVisitor<'a> {
    source: &'a str,
    collected: CollectedOrigins,
    csp_depth: usize,
    in_csp_fn: bool,
    resolved_bindings: HashSet<String>,
}

impl<'a> Visit<'a> for OriginVisitor<'_> {
    fn visit_ts_type(&mut self, _it: &TSType<'a>) {}

    fn visit_string_literal(&mut self, lit: &StringLiteral<'a>) {
        if self.csp_depth == 0 {
            let value = crate::ast::js_literal(lit);
            if is_static_csp_policy_string(&value) {
                self.collected.csp_strings.push(value.into_owned());
            }
        }
    }

    fn visit_template_literal(&mut self, tpl: &TemplateLiteral<'a>) {
        self.visit_csp_expr(
            |out| try_collect_template(tpl, out),
            |this| walk::walk_template_literal(this, tpl),
        );
    }

    fn visit_array_expression(&mut self, array: &ArrayExpression<'a>) {
        self.visit_csp_expr(
            |out| try_collect_array(array, out),
            |this| walk::walk_array_expression(this, array),
        );
    }

    fn visit_object_expression(&mut self, obj: &ObjectExpression<'a>) {
        self.visit_csp_expr(
            |_| try_collect_object(obj),
            |this| walk::walk_object_expression(this, obj),
        );
    }

    fn visit_binary_expression(&mut self, expr: &BinaryExpression<'a>) {
        self.visit_csp_expr(
            |out| try_collect_binary(expr, out),
            |this| walk::walk_binary_expression(this, expr),
        );
    }

    fn visit_conditional_expression(&mut self, expr: &ConditionalExpression<'a>) {
        self.visit_csp_expr(
            |_| try_collect_conditional(expr),
            |this| walk::walk_conditional_expression(this, expr),
        );
    }

    fn visit_logical_expression(&mut self, expr: &LogicalExpression<'a>) {
        self.visit_csp_expr(
            |_| try_collect_logical(expr),
            |this| walk::walk_logical_expression(this, expr),
        );
    }

    fn visit_jsx_opening_element(&mut self, element: &JSXOpeningElement<'a>) {
        record_jsx_element(&mut self.collected, self.source, element);
        walk::walk_jsx_opening_element(self, element);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_global_or_bare(&call.callee, &["fetch"]) {
            self.record_first_arg(&call.arguments, CspDirective::Connect);
        }
        self.visit_csp_expr(
            |out| try_collect_call(call, out),
            |this| walk::walk_call_expression(this, call),
        );
    }

    fn visit_new_expression(&mut self, expr: &NewExpression<'a>) {
        if is_global_or_bare(&expr.callee, &["WebSocket", "EventSource"]) {
            self.record_first_arg(&expr.arguments, CspDirective::Connect);
        }
        walk::walk_new_expression(self, expr);
    }

    fn visit_function(&mut self, func: &Function<'a>, flags: ScopeFlags) {
        let name = func.id.as_ref().map(|id| id.name.as_str());
        let is_csp = self.csp_depth == 0 && name.is_some_and(is_csp_identifier);
        let prev = std::mem::replace(&mut self.in_csp_fn, is_csp);
        let (s0, u0) = self.counts();
        walk::walk_function(self, func, flags);
        self.in_csp_fn = prev;
        if let Some(name) = name.filter(|_| is_csp) {
            self.note_resolved_binding(name, s0, u0);
        }
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        let prev = std::mem::replace(&mut self.in_csp_fn, false);
        walk::walk_arrow_function_expression(self, arrow);
        self.in_csp_fn = prev;
    }

    fn visit_return_statement(&mut self, ret: &ReturnStatement<'a>) {
        if self.in_csp_fn && self.csp_depth == 0 {
            if let Some(arg) = &ret.argument {
                self.walk_csp_value_expr(arg);
                return;
            }
        }
        walk::walk_return_statement(self, ret);
    }

    fn visit_variable_declarator(&mut self, decl: &VariableDeclarator<'a>) {
        let BindingPattern::BindingIdentifier(id) = &decl.id else {
            walk::walk_variable_declarator(self, decl);
            return;
        };
        let Some(init) = &decl.init else {
            return;
        };
        let name = id.name.as_str();
        if self.csp_depth != 0 || !is_csp_identifier(name) {
            walk::walk_variable_declarator(self, decl);
            return;
        }
        let (s0, u0) = self.counts();
        match unwrap_ts_wrappers(init) {
            Expression::ArrowFunctionExpression(arrow) => self.walk_csp_arrow(arrow),
            Expression::FunctionExpression(func) => {
                let prev = std::mem::replace(&mut self.in_csp_fn, true);
                walk::walk_function(self, func, ScopeFlags::Function);
                self.in_csp_fn = prev;
            }
            _ => self.walk_csp_value_expr(init),
        }
        self.note_resolved_binding(name, s0, u0);
    }
}
