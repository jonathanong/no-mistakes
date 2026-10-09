use super::{Roots, SqlFunctionClause};
use sqlparser::ast::{
    Function, FunctionArgumentClause, FunctionArguments, NamedWindowExpr, OrderBy, OrderByKind,
    Query, Select, SetExpr, WindowFrameBound, WindowSpec, WindowType,
};

impl Roots {
    pub(in crate::codebase::postgres::function_calls) fn query(&mut self, query: &Query) {
        self.values(&query.body);
        if let Some(order_by) = &query.order_by {
            self.order_by(order_by);
        }
    }

    fn values(&mut self, body: &SetExpr) {
        match body {
            SetExpr::Values(values) => {
                for row in &values.rows {
                    for expression in row.iter() {
                        self.expr(expression, SqlFunctionClause::Values);
                    }
                }
            }
            SetExpr::SetOperation { left, right, .. } => {
                self.values(left);
                self.values(right);
            }
            // SELECT and nested query roots are registered by their own visitor hooks.
            _ => {}
        }
    }

    pub(in crate::codebase::postgres::function_calls) fn select(
        &mut self,
        select: &Select,
        projection: bool,
    ) {
        if projection {
            self.items(&select.projection, SqlFunctionClause::SelectList);
        }
        if let Some(expression) = &select.selection {
            self.expr(expression, SqlFunctionClause::Where);
        }
        if let Some(expression) = &select.having {
            self.expr(expression, SqlFunctionClause::Having);
        }
        self.joins(&select.from);
        for window in &select.named_window {
            if let NamedWindowExpr::WindowSpec(spec) = &window.1 {
                self.window(spec);
            }
        }
    }

    pub(in crate::codebase::postgres::function_calls) fn function(&mut self, function: &Function) {
        if let Some(expression) = &function.filter {
            self.expr(expression, SqlFunctionClause::Where);
        }
        self.order_exprs(&function.within_group);
        if let Some(WindowType::WindowSpec(spec)) = &function.over {
            self.window(spec);
        }
        for arguments in [&function.parameters, &function.args] {
            if let FunctionArguments::List(list) = arguments {
                for clause in &list.clauses {
                    match clause {
                        FunctionArgumentClause::Where(expression) => {
                            self.expr(expression, SqlFunctionClause::Where);
                        }
                        FunctionArgumentClause::OrderBy(expressions) => {
                            self.order_exprs(expressions);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn window(&mut self, spec: &WindowSpec) {
        for expression in &spec.partition_by {
            self.unscoped(expression);
        }
        if let Some(frame) = &spec.window_frame {
            for bound in std::iter::once(&frame.start_bound).chain(frame.end_bound.as_ref()) {
                if let WindowFrameBound::Preceding(Some(expr))
                | WindowFrameBound::Following(Some(expr)) = bound
                {
                    self.unscoped(expr);
                }
            }
        }
        self.order_exprs(&spec.order_by);
    }

    pub(in crate::codebase::postgres::function_calls) fn order_by(&mut self, order_by: &OrderBy) {
        if let OrderByKind::Expressions(expressions) = &order_by.kind {
            self.order_exprs(expressions);
        }
    }
}
