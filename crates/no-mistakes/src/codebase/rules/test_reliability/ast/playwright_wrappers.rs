use super::bindings::Bindings;
use crate::codebase::rules::test_reliability::playwright_provenance::{WrapperCall, WrapperSummary};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{Expression, FormalParameters, FunctionBody, Statement};
use oxc_ast::AstKind;
use oxc_semantic::SymbolId;

const MAX_WRAPPER_DEPTH: usize = 8;

enum WrapperBody<'ast> {
    Expr(&'ast Expression<'ast>),
    Block(&'ast FunctionBody<'ast>),
}

impl<'ast> Bindings<'_, 'ast> {
    pub(super) fn wrapper_summary(
        &self,
        symbol: SymbolId,
        depth: usize,
        visited: &mut FxHashSet<SymbolId>,
    ) -> Option<WrapperSummary> {
        if depth >= MAX_WRAPPER_DEPTH || !visited.insert(symbol) {
            return None;
        }
        let result = self.analyze_wrapper(symbol, depth, visited);
        visited.remove(&symbol);
        result
    }

    fn analyze_wrapper(
        &self,
        symbol: SymbolId,
        depth: usize,
        visited: &mut FxHashSet<SymbolId>,
    ) -> Option<WrapperSummary> {
        let (formal_params, body) = self.function_def(symbol)?;
        let params: FxHashMap<SymbolId, usize> = formal_params
            .items
            .iter()
            .enumerate()
            .filter_map(|(i, item)| {
                Some((item.pattern.get_binding_identifier()?.symbol_id.get()?, i))
            })
            .collect();
        let mut returns = Vec::new();
        match body {
            WrapperBody::Expr(expr) => returns.push(expr),
            WrapperBody::Block(block) => {
                if !collect_returns(&block.statements, &mut returns) || returns.is_empty() {
                    return None;
                }
            }
        }
        let mut param_index = None;
        let mut wrappers = Vec::new();
        let mut derived_at = None;
        for expr in returns {
            let branch = self.param_origin(expr, &params, depth, visited)?;
            if param_index.is_some_and(|prev| prev != branch.param_index) {
                return None;
            }
            param_index = Some(branch.param_index);
            extend_unique(&mut wrappers, branch.wrappers);
            derived_at = derived_at.or(branch.derived_at);
        }
        Some(WrapperSummary {
            param_index: param_index?,
            wrappers,
            derived_at,
        })
    }

    fn param_origin(
        &self,
        expr: &'ast Expression<'ast>,
        params: &FxHashMap<SymbolId, usize>,
        depth: usize,
        visited: &mut FxHashSet<SymbolId>,
    ) -> Option<WrapperSummary> {
        let mut value = unwrap_ts_wrappers(expr);
        let mut wrappers = Vec::new();
        let mut derived_at = None;
        loop {
            match value {
                Expression::Identifier(identifier) => {
                    if let Some(&param_index) = params.get(&self.symbol(identifier)?) {
                        return Some(WrapperSummary {
                            param_index,
                            wrappers,
                            derived_at,
                        });
                    }
                    value = unwrap_ts_wrappers(self.constant_initializer(identifier)?);
                }
                Expression::CallExpression(call) if !call.optional => {
                    let callee = unwrap_ts_wrappers(&call.callee);
                    if let Some(member) = callee.as_member_expression() {
                        if member.static_property_name().and_then(|n| n.as_str()) == Some("extend")
                        {
                            derived_at.get_or_insert(call.span.start);
                            value = unwrap_ts_wrappers(member.object());
                            continue;
                        }
                    }
                    if let Some(inner) = self.local_callee_summary(callee, depth + 1, visited) {
                        extend_unique(&mut wrappers, inner.wrappers);
                        derived_at = derived_at.or(inner.derived_at);
                        value = unwrap_ts_wrappers(
                            call.arguments.get(inner.param_index)?.as_expression()?,
                        );
                        continue;
                    }
                    let import = self.callee_import_origin(callee)?;
                    for (arg_index, arg) in call.arguments.iter().enumerate() {
                        let Some(arg_expr) = arg.as_expression() else {
                            continue;
                        };
                        if let Some(mut inner) =
                            self.param_origin(arg_expr, params, depth + 1, visited)
                        {
                            inner.wrappers.push(WrapperCall {
                                callee: import,
                                arg_index,
                            });
                            extend_unique(&mut inner.wrappers, wrappers);
                            inner.derived_at = derived_at.or(inner.derived_at);
                            return Some(inner);
                        }
                    }
                    return None;
                }
                _ => return None,
            }
        }
    }

    fn function_def(&self, symbol: SymbolId) -> Option<(&'ast FormalParameters<'ast>, WrapperBody<'ast>)> {
        let scope = self.semantic.scoping();
        match self.semantic.nodes().kind(scope.symbol_declaration(symbol)) {
            AstKind::Function(func) => Some((&func.params, WrapperBody::Block(func.body.as_deref()?))),
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
}

fn extend_unique(target: &mut Vec<WrapperCall>, extra: Vec<WrapperCall>) {
    for item in extra {
        if !target.contains(&item) {
            target.push(item);
        }
    }
}

fn collect_returns<'ast>(stmts: &'ast [Statement<'ast>], out: &mut Vec<&'ast Expression<'ast>>) -> bool {
    for stmt in stmts {
        match stmt {
            Statement::ReturnStatement(ret) => {
                let Some(arg) = ret.argument.as_ref() else { return false };
                out.push(arg);
            }
            Statement::BlockStatement(block) if !collect_returns(&block.body, out) => return false,
            Statement::IfStatement(if_stmt)
                if !collect_returns(std::slice::from_ref(&if_stmt.consequent), out)
                    || if_stmt
                        .alternate
                        .as_ref()
                        .is_some_and(|alt| !collect_returns(std::slice::from_ref(alt), out)) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}
