use super::names::{Names, ValueKind};
use crate::fx::FxHashSet;
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

pub(super) fn called_names(program: &Program<'_>) -> FxHashSet<String> {
    let mut scan = CalledNames::default();
    scan.visit_program(program);
    scan.names
}

pub(super) fn record_function(
    next: &mut Names,
    prior: &Names,
    function: &Function<'_>,
    called: &FxHashSet<String>,
) {
    let Some(name) = function.id.as_ref().map(|id| id.name.as_str()) else {
        return;
    };
    if !called.contains(name) {
        return;
    }
    next.record_call_result(name, function_result(prior, function));
}

pub(super) fn record_expression(
    next: &mut Names,
    prior: &Names,
    name: &str,
    init: &Expression<'_>,
    called: &FxHashSet<String>,
) {
    if !called.contains(name) {
        return;
    }
    let kind = match init.without_parentheses() {
        Expression::FunctionExpression(function) => function_result(prior, function),
        Expression::ArrowFunctionExpression(arrow) => arrow_result(prior, arrow),
        _ => return,
    };
    next.record_call_result(name, kind);
}

fn function_result(names: &Names, function: &Function<'_>) -> Option<ValueKind> {
    if function.generator {
        return None;
    }
    if function.r#async {
        return Some(ValueKind::Promise);
    }
    if let Some(ret) = &function.return_type {
        return names.type_kind(&ret.type_annotation);
    }
    function
        .body
        .as_deref()
        .and_then(|body| body_result(names, body))
}

fn arrow_result(names: &Names, arrow: &ArrowFunctionExpression<'_>) -> Option<ValueKind> {
    if arrow.r#async {
        return Some(ValueKind::Promise);
    }
    if let Some(ret) = &arrow.return_type {
        return names.type_kind(&ret.type_annotation);
    }
    if let Some(expression) = arrow.body.as_expression() {
        return names.value_kind(expression, 0);
    }
    crate::ast::arrow_function_body(&arrow.body).and_then(|body| body_result(names, body))
}

fn body_result(names: &Names, body: &FunctionBody<'_>) -> Option<ValueKind> {
    let mut returns = Returns {
        names,
        kinds: Vec::new(),
    };
    returns.visit_function_body(body);
    let first = returns.kinds.first().copied().flatten()?;
    returns
        .kinds
        .iter()
        .all(|kind| *kind == Some(first))
        .then_some(first)
}

#[derive(Default)]
struct CalledNames {
    names: FxHashSet<String>,
}

impl<'a> Visit<'a> for CalledNames {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(id) = call.callee.without_parentheses() {
            self.names.insert(id.name.to_string());
        }
        walk::walk_call_expression(self, call);
    }
}

struct Returns<'a> {
    names: &'a Names,
    kinds: Vec<Option<ValueKind>>,
}

impl<'a> Visit<'a> for Returns<'_> {
    fn visit_function(&mut self, _function: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}

    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}

    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        self.kinds.push(
            stmt.argument
                .as_ref()
                .and_then(|expr| self.names.value_kind(expr, 0)),
        );
    }
}
