use super::super::bindings::Bindings;
use super::super::playwright_bindings::ValueTrace;
use super::extend_unique;
use super::returns::collect_returns;
use crate::codebase::rules::test_reliability::playwright_provenance::{
    FixtureInitializer, WrapperCall,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{CallExpression, Expression, FormalParameters, FunctionBody};
use oxc_ast::AstKind;
use oxc_semantic::SymbolId;
pub(super) enum WrapperBody<'ast> {
    Expr(&'ast Expression<'ast>),
    Block(&'ast FunctionBody<'ast>),
}

impl<'ast> Bindings<'_, 'ast> {
    pub(in super::super) fn is_function_def(&self, symbol: SymbolId) -> bool {
        self.function_def(symbol).is_some()
    }

    pub(super) fn function_def(
        &self,
        symbol: SymbolId,
    ) -> Option<(&'ast FormalParameters<'ast>, WrapperBody<'ast>)> {
        let scope = self.semantic.scoping();
        match self.semantic.nodes().kind(scope.symbol_declaration(symbol)) {
            AstKind::Function(func) => {
                Some((&func.params, WrapperBody::Block(func.body.as_deref()?)))
            }
            AstKind::VariableDeclarator(decl)
                if scope.symbol_flags(symbol).is_const_variable()
                    && decl.id.is_binding_identifier() =>
            {
                match unwrap_ts_wrappers(decl.init.as_ref()?) {
                    Expression::ArrowFunctionExpression(arrow) => {
                        let body = arrow
                            .get_expression()
                            .map(WrapperBody::Expr)
                            .or_else(|| arrow.get_function_body().map(WrapperBody::Block))?;
                        Some((&arrow.params, body))
                    }
                    Expression::FunctionExpression(func) => {
                        Some((&func.params, WrapperBody::Block(func.body.as_deref()?)))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    pub(in crate::codebase::rules::test_reliability::ast) fn factory_return_origin(
        &self,
        symbol: SymbolId,
        depth: usize,
    ) -> ValueTrace {
        let Some((_, body)) = self.function_def(symbol) else {
            return ValueTrace::None;
        };
        let mut returns = Vec::new();
        match body {
            WrapperBody::Expr(expr) => returns.push(expr),
            WrapperBody::Block(block) => {
                if !collect_returns(&block.statements, &mut returns) || returns.is_empty() {
                    return self.unresolved_if_body_provenance(
                        block,
                        "local factory return could not be traced statically",
                    );
                }
            }
        }
        let mut candidates = Vec::new();
        let mut reason = None;
        let mut wrappers = Vec::new();
        let mut derived_at = None;
        for expr in returns {
            let branch = match self.playwright_value_origin_depth(expr, depth + 1) {
                ValueTrace::Resolved(init) => init,
                ValueTrace::Unresolved(branch_reason, imports) => {
                    reason.get_or_insert(branch_reason);
                    candidates.extend(imports);
                    continue;
                }
                ValueTrace::None => {
                    reason.get_or_insert("local factory return could not be traced statically");
                    continue;
                }
            };
            if !candidates.contains(&branch.import) {
                candidates.push(branch.import);
            }
            extend_unique(&mut wrappers, branch.wrappers);
            derived_at = derived_at.or(branch.derived_at);
        }
        match candidates.pop() {
            Some(import) if reason.is_none() && candidates.is_empty() => {
                ValueTrace::Resolved(FixtureInitializer {
                    import,
                    wrappers,
                    derived_at,
                })
            }
            Some(import) => {
                candidates.push(import);
                ValueTrace::unresolved(
                    reason.unwrap_or("local factory returns multiple distinct fixture origins"),
                    candidates,
                )
            }
            None => ValueTrace::None,
        }
    }

    pub(in crate::codebase::rules::test_reliability::ast) fn call_value_origin(
        &self,
        call: &CallExpression<'_>,
        callee: &Expression<'_>,
        depth: usize,
        wrappers: Vec<WrapperCall>,
        derived_at: Option<u32>,
    ) -> ValueTrace {
        let Some(import) = self.callee_import_origin(callee) else {
            if let Expression::Identifier(id) = callee {
                if let Some(symbol) = self.symbol(id) {
                    match self.factory_return_origin(symbol, depth) {
                        ValueTrace::Resolved(mut inner) => {
                            extend_unique(&mut inner.wrappers, wrappers);
                            inner.derived_at = derived_at.or(inner.derived_at);
                            return ValueTrace::Resolved(inner);
                        }
                        trace @ ValueTrace::Unresolved(..) => return trace,
                        ValueTrace::None => {}
                    }
                }
            }
            return self.unresolved_if_call_provenance(
                call,
                "local wrapper or call in fixture chain could not be traced statically",
            );
        };
        for (arg_index, arg) in call.arguments.iter().enumerate() {
            let Some(arg_expr) = arg.as_expression() else {
                continue;
            };
            match self.playwright_value_origin_depth(arg_expr, depth + 1) {
                ValueTrace::Resolved(mut inner) => {
                    inner.wrappers.push(WrapperCall {
                        callee: import,
                        arg_index,
                    });
                    extend_unique(&mut inner.wrappers, wrappers);
                    inner.derived_at = derived_at.or(inner.derived_at);
                    return ValueTrace::Resolved(inner);
                }
                ValueTrace::Unresolved(reason, mut candidates) => {
                    candidates.push(import);
                    return ValueTrace::Unresolved(reason, candidates);
                }
                ValueTrace::None => {}
            }
        }
        ValueTrace::Resolved(FixtureInitializer {
            import,
            wrappers,
            derived_at,
        })
    }
}
