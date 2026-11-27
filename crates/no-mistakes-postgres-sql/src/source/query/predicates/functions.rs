use super::*;
use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, InTarget, WindowType,
};
impl Collector<'_, '_> {
    pub(super) fn in_expression(
        &mut self,
        expr: &Expr,
        target: &InTarget,
        scope: usize,
        clause: PostgresSqlQueryClause,
        context: PostgresSqlPredicateContext,
        env: &CteEnvironment,
    ) -> bool {
        let child = super::opaque_context(context);
        match target {
            InTarget::Subquery(subquery) => {
                self.expr(expr, scope, clause, None, child, env);
                self.query(
                    subquery,
                    Some(scope),
                    Some(scope),
                    clause,
                    env,
                    self.facts.scopes[scope].cte_definition_id,
                );
                true
            }
            InTarget::List(list) => {
                self.expr(expr, scope, clause, None, child.clone(), env);
                for item in list {
                    self.expr(item, scope, clause, None, child.clone(), env);
                }
                true
            }
            InTarget::Unnest(_) | InTarget::Table { .. } => false,
        }
    }

    pub(in super::super) fn function_expressions(
        &mut self,
        function: &Function,
        scope: usize,
        clause: PostgresSqlQueryClause,
        join: Option<usize>,
        mut context: PostgresSqlPredicateContext,
        env: &CteEnvironment,
    ) {
        context.mandatory = false;
        context.under_other = true;
        context.effective_mandatory = None;
        for arguments in [&function.parameters, &function.args] {
            match arguments {
                FunctionArguments::None => {}
                FunctionArguments::Subquery(query) => {
                    self.query(
                        query,
                        Some(scope),
                        Some(scope),
                        clause,
                        env,
                        self.facts.scopes[scope].cte_definition_id,
                    );
                }
                FunctionArguments::List(list) => {
                    for arg in &list.args {
                        let value = match arg {
                            FunctionArg::Unnamed(arg)
                            | FunctionArg::Named { arg, .. }
                            | FunctionArg::ExprNamed { arg, .. } => arg,
                        };
                        if let FunctionArgExpr::Expr(expr) = value {
                            self.expr(expr, scope, clause, join, context.clone(), env);
                        }
                    }
                    if !list.clauses.is_empty() {
                        self.unsupported(
                            scope,
                            clause,
                            "function argument clause",
                            function.span(),
                        );
                    }
                }
            }
        }
        if let Some(expr) = &function.filter {
            self.expr(expr, scope, clause, join, context.clone(), env);
        }
        for order in &function.within_group {
            self.expr(&order.expr, scope, clause, join, context.clone(), env);
        }
        if let Some(over) = &function.over {
            match over {
                WindowType::WindowSpec(spec) => {
                    for expr in &spec.partition_by {
                        self.expr(expr, scope, clause, join, context.clone(), env);
                    }
                    for order in &spec.order_by {
                        self.expr(&order.expr, scope, clause, join, context.clone(), env);
                    }
                    if spec.window_frame.is_some() {
                        self.unsupported(scope, clause, "window frame", function.span());
                    }
                }
                WindowType::NamedWindow(_) => {
                    self.unsupported(scope, clause, "named window", function.span())
                }
            }
        }
    }
}
