use super::{Expr, Scope};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Argument, ArrowFunctionBody, Expression, FunctionBody, Statement};
use oxc_ast_visit::{walk, Visit};

pub(super) fn method_argument(scope: &mut Scope<'_>, argument: &Argument<'_>) -> Expr {
    let Some(expression) = argument.as_expression() else {
        return Expr::Opaque;
    };
    match unwrap_ts_wrappers(expression) {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow_carries_sql(&arrow.body) {
                Expr::Opaque
            } else {
                Expr::Num
            }
        }
        Expression::FunctionExpression(function) => {
            if function.body.as_deref().is_some_and(body_carries_sql) {
                Expr::Opaque
            } else {
                Expr::Num
            }
        }
        _ => scope.expression(expression),
    }
}

fn arrow_carries_sql(body: &ArrowFunctionBody<'_>) -> bool {
    if let Some(expression) = body.as_expression() {
        return carries_sql(expression);
    }
    let ArrowFunctionBody::FunctionBody(body) = body else {
        return true;
    };
    body_carries_sql(body)
}

fn body_carries_sql(body: &FunctionBody<'_>) -> bool {
    body.statements.iter().any(statement_carries_sql)
}

fn statement_carries_sql(statement: &Statement<'_>) -> bool {
    let mut probe = Probe(false);
    probe.visit_statement(statement);
    probe.0
}

fn carries_sql(expression: &Expression<'_>) -> bool {
    let mut probe = Probe(false);
    probe.visit_expression(expression);
    probe.0
}

struct Probe(bool);

impl<'a> Visit<'a> for Probe {
    fn visit_call_expression(&mut self, node: &oxc_ast::ast::CallExpression<'a>) {
        self.0 = true;
        walk::walk_call_expression(self, node);
    }
    fn visit_template_literal(&mut self, node: &oxc_ast::ast::TemplateLiteral<'a>) {
        self.0 = true;
        walk::walk_template_literal(self, node);
    }
    fn visit_tagged_template_expression(
        &mut self,
        node: &oxc_ast::ast::TaggedTemplateExpression<'a>,
    ) {
        self.0 = true;
        walk::walk_tagged_template_expression(self, node);
    }
}
