use super::bindings::Bindings;
use crate::codebase::rules::test_reliability::playwright_provenance::{WrapperCall, WrapperSummary};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{CallExpression, Expression};
use oxc_semantic::SymbolId;

mod body;
use body::{collect_returns, WrapperBody};

const MAX_WRAPPER_DEPTH: usize = 8;

struct ParamSearch<'a> {
    params: &'a FxHashMap<SymbolId, usize>,
    depth: usize,
    visited: &'a mut FxHashSet<SymbolId>,
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
        let mut search = ParamSearch {
            params: &params,
            depth,
            visited,
        };
        for expr in returns {
            let branch = self.param_origin(expr, &mut search)?;
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
        search: &mut ParamSearch<'_>,
    ) -> Option<WrapperSummary> {
        let mut value = unwrap_ts_wrappers(expr);
        let mut wrappers = Vec::new();
        let mut derived_at = None;
        loop {
            match value {
                Expression::Identifier(identifier) => {
                    if let Some(&param_index) = search.params.get(&self.symbol(identifier)?) {
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
                    if let Some(next) = self.unwrap_local_call(
                        call,
                        callee,
                        search,
                        &mut wrappers,
                        &mut derived_at,
                    ) {
                        value = next;
                        continue;
                    }
                    return self.imported_param_call(call, callee, search, wrappers, derived_at);
                }
                _ => return None,
            }
        }
    }

    fn unwrap_local_call(
        &self,
        call: &'ast CallExpression<'ast>,
        callee: &'ast Expression<'ast>,
        search: &mut ParamSearch<'_>,
        wrappers: &mut Vec<WrapperCall>,
        derived_at: &mut Option<u32>,
    ) -> Option<&'ast Expression<'ast>> {
        if let Some(member) = callee.as_member_expression() {
            if member.static_property_name().and_then(|n| n.as_str()) == Some("extend") {
                derived_at.get_or_insert(call.span.start);
                return Some(unwrap_ts_wrappers(member.object()));
            }
        }
        let inner = self.local_callee_summary(callee, search.depth + 1, search.visited)?;
        extend_unique(wrappers, inner.wrappers);
        *derived_at = derived_at.or(inner.derived_at);
        Some(unwrap_ts_wrappers(
            call.arguments.get(inner.param_index)?.as_expression()?,
        ))
    }

    fn imported_param_call(
        &self,
        call: &'ast CallExpression<'ast>,
        callee: &'ast Expression<'ast>,
        search: &mut ParamSearch<'_>,
        wrappers: Vec<WrapperCall>,
        derived_at: Option<u32>,
    ) -> Option<WrapperSummary> {
        let import = self.callee_import_origin(callee)?;
        let mut nested = ParamSearch {
            params: search.params,
            depth: search.depth + 1,
            visited: search.visited,
        };
        for (arg_index, arg) in call.arguments.iter().enumerate() {
            let Some(arg_expr) = arg.as_expression() else {
                continue;
            };
            if let Some(mut inner) = self.param_origin(arg_expr, &mut nested) {
                inner.wrappers.push(WrapperCall {
                    callee: import,
                    arg_index,
                });
                extend_unique(&mut inner.wrappers, wrappers);
                inner.derived_at = derived_at.or(inner.derived_at);
                return Some(inner);
            }
        }
        None
    }
}

pub(super) fn extend_unique(target: &mut Vec<WrapperCall>, extra: Vec<WrapperCall>) {
    for item in extra {
        if !target.contains(&item) {
            target.push(item);
        }
    }
}
