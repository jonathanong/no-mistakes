use super::{record_qualified, BaseRel, SqlStarProjectionFact};
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, Spanned};

pub(super) fn walk_expr(
    expr: &Expr,
    rels: &[BaseRel],
    line: usize,
    out: &mut Vec<SqlStarProjectionFact>,
) {
    use sqlparser::ast::{Visit, Visitor};
    struct Functions<'a> {
        rels: &'a [BaseRel],
        line: usize,
        out: &'a mut Vec<SqlStarProjectionFact>,
        query_depth: usize,
    }
    impl Visitor for Functions<'_> {
        type Break = ();
        fn pre_visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<()> {
            if self.query_depth == 0 {
                if let Expr::Function(function) = expr {
                    walk_function(
                        function,
                        self.rels,
                        span_line(expr.span(), self.line),
                        self.out,
                    );
                }
            }
            std::ops::ControlFlow::Continue(())
        }
        fn pre_visit_query(&mut self, _: &sqlparser::ast::Query) -> std::ops::ControlFlow<()> {
            self.query_depth += 1;
            std::ops::ControlFlow::Continue(())
        }
        fn post_visit_query(&mut self, _: &sqlparser::ast::Query) -> std::ops::ControlFlow<()> {
            self.query_depth -= 1;
            std::ops::ControlFlow::Continue(())
        }
    }
    let _ = expr.visit(&mut Functions {
        rels,
        line,
        out,
        query_depth: 0,
    });
}

fn walk_function(
    function: &Function,
    rels: &[BaseRel],
    line: usize,
    out: &mut Vec<SqlStarProjectionFact>,
) {
    let FunctionArguments::List(list) = &function.args else {
        return;
    };
    let name = relation_name(&function.name).to_ascii_lowercase();
    for arg in &list.args {
        let expr = arg_expr(arg);
        match expr {
            FunctionArgExpr::QualifiedWildcard(object) => {
                record_qualified(
                    rels,
                    object,
                    Some(name.clone()),
                    span_line(object.span(), line),
                    out,
                );
            }
            FunctionArgExpr::Expr(_) => {}
            FunctionArgExpr::Wildcard | FunctionArgExpr::WildcardWithOptions(_) => {}
        }
    }
}

pub(super) fn span_line(span: sqlparser::tokenizer::Span, fallback: usize) -> usize {
    if span.start == span.end || span.start.line == 0 {
        fallback.max(1)
    } else {
        span.start.line as usize
    }
}

pub(in crate::codebase::postgres::statements) fn arg_expr(arg: &FunctionArg) -> &FunctionArgExpr {
    match arg {
        FunctionArg::Unnamed(expr)
        | FunctionArg::Named { arg: expr, .. }
        | FunctionArg::ExprNamed { arg: expr, .. } => expr,
    }
}
