use super::uses::Uses;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, BindingPattern, Expression};
use oxc_ast_visit::Visit;
mod parse;

pub(in crate::codebase::postgres::embedded::walk) struct FiniteCase {
    alias: String,
    values: Vec<(String, u32)>,
    depth: usize,
    pub(in crate::codebase::postgres::embedded::walk) decision: u64,
}
impl FiniteCase {
    pub(in crate::codebase::postgres::embedded::walk) fn from_call(
        call: &ast::CallExpression<'_>,
        imports: &FxHashSet<String>,
        shadowed: impl Fn(&str) -> bool,
        depth: usize,
    ) -> Option<Self> {
        let keys = parse::keys(call, imports, shadowed)?;
        let Expression::ArrowFunctionExpression(callback) =
            unwrap_ts_wrappers(call.arguments.get(1)?.as_expression()?)
        else {
            return None;
        };
        if callback.params.rest.is_some() || callback.params.items.len() != 1 {
            return None;
        }
        let param = &callback.params.items[0];
        if param.initializer.is_some() || !param.decorators.is_empty() {
            return None;
        }
        let BindingPattern::BindingIdentifier(key) = &param.pattern else {
            return None;
        };
        let ast::ArrowFunctionBody::FunctionBody(body) = &callback.body else {
            return None;
        };
        let selected = parse::selection(body, key.name.as_str(), keys)?;
        let mut uses = Uses {
            map: &selected.map,
            alias: &selected.alias,
            key: key.name.as_str(),
            allowed: selected.allowed,
            declarations: selected.declarations,
            valid: true,
            nodes: 0,
        };
        uses.visit_function_body(body);
        uses.valid.then_some(Self {
            alias: selected.alias,
            values: selected.values,
            depth: depth + 1,
            decision: u64::from(call.span.start) | (1u64 << 62),
        })
    }
    pub(in crate::codebase::postgres::embedded::walk) fn values(
        &self,
        name: &str,
        depth: usize,
    ) -> Option<(&[(String, u32)], u64)> {
        (self.alias == name && self.depth == depth).then_some((&self.values, self.decision))
    }
}
