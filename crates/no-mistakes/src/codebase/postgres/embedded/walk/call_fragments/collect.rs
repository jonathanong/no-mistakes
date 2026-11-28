use super::super::resolve::compose;
use super::super::ScopeVisitor;
use super::{partial, BuilderTrace};
use crate::codebase::postgres::embedded::{
    first_call_argument, tags, CallSqlFragment, EmbeddedSqlCall,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{BinaryOperator, CallExpression, Expression};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn pop_builder_fragments(
        &mut self,
        scope_index: usize,
        scope_id: Option<u32>,
    ) {
        self.call_fragment_state
            .builders
            .retain(|(scope, _), _| *scope != scope_index);
        if let Some(id) = scope_id {
            self.call_fragment_state
                .function_returns
                .retain(|(owner_scope, _), _| *owner_scope != id);
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn note_returned_builder_fragments(
        &mut self,
        expr: &Expression<'_>,
    ) {
        if self.head_calls_only {
            return;
        }
        let Some(owner) = self.current_plain_function_key() else {
            return;
        };
        let returned = match unwrap_ts_wrappers(expr) {
            Expression::Identifier(id) => self
                .lookup_scope(id.name.as_str())
                .and_then(|s| {
                    self.call_fragment_state
                        .builders
                        .get(&(s, id.name.to_string()))
                })
                .cloned(),
            Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
                let mut fragments = Vec::new();
                collect_return_operand_traces(self, expr, &mut fragments);
                fragments.extend(partial::expr_fragments(self, expr, None, None));
                (!fragments.is_empty()).then(|| BuilderTrace::from_fragments(fragments))
            }
            _ if partial::is_sql_like_init(self, expr) => {
                let fragments = partial::expr_fragments(self, expr, None, None);
                (!fragments.is_empty()).then(|| BuilderTrace::from_fragments(fragments))
            }
            _ => None,
        };
        if let Some(trace) = returned {
            self.call_fragment_state
                .function_returns
                .entry(owner)
                .or_default()
                .merge(trace);
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn collect_call_fragments(
        &self,
        call: &CallExpression<'_>,
        recovered: &EmbeddedSqlCall,
    ) -> Vec<CallSqlFragment> {
        if self.head_calls_only
            || (!recovered.is_unanalyzable() && !super::inline_append::call_has_sentinel(recovered))
        {
            return Vec::new();
        }
        let covered_sites: FxHashSet<u32> = recovered
            .variants
            .iter()
            .filter(|variant| !variant.sql_text.contains("sql_placeholder_"))
            .flat_map(|variant| variant.append_sites.iter().copied())
            .collect();
        let variants_cover_init = !recovered.variants.is_empty()
            && !recovered
                .variants
                .iter()
                .any(|v| v.sql_text.contains("sql_placeholder_"));
        let Some(arg) = first_call_argument(call) else {
            return Vec::new();
        };
        match unwrap_ts_wrappers(arg) {
            Expression::Identifier(id) => {
                if let Some(scope_index) = self.lookup_scope(id.name.as_str()) {
                    if let Some(trace) = self
                        .call_fragment_state
                        .builders
                        .get(&(scope_index, id.name.to_string()))
                    {
                        return trace
                            .fragments
                            .iter()
                            .filter(|fragment| {
                                let covered = fragment
                                    .append_site
                                    .is_some_and(|site| covered_sites.contains(&site));
                                let duplicate_init =
                                    fragment.append_site.is_none() && variants_cover_init;
                                !covered && !duplicate_init
                            })
                            .cloned()
                            .collect();
                    }
                }
                if recovered.variants.is_empty() {
                    if let Some(sql) = &recovered.sql_text {
                        return vec![CallSqlFragment {
                            sql: sql.clone(),
                            line: recovered.declaration_line.unwrap_or(recovered.line),
                            span: (call.span.start, call.span.end),
                            context_sql: None,
                            append_site: None,
                        }];
                    }
                }
                Vec::new()
            }
            Expression::CallExpression(inner) => {
                if let Expression::Identifier(callee) = unwrap_ts_wrappers(&inner.callee) {
                    return self
                        .call_fragment_state
                        .function_returns
                        .get(&self.plain_key(callee.name.as_str()))
                        .map(|trace| trace.fragments.clone())
                        .unwrap_or_default();
                }
                if let Some(frags) = super::inline_append::inline_append_fragments(
                    self,
                    inner,
                    &covered_sites,
                    variants_cover_init,
                ) {
                    return frags;
                }
                if recovered.variants.is_empty() {
                    partial::expr_fragments(self, arg, None, None)
                } else {
                    Vec::new()
                }
            }
            _ if recovered.variants.is_empty() => partial::expr_fragments(self, arg, None, None),
            _ => Vec::new(),
        }
    }

    pub(super) fn lookup_scope(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rposition(|scope| scope.contains_key(name))
    }

    pub(super) fn is_tagged_sql_fragment(&self, expr: &Expression<'_>) -> bool {
        tags::fragments::is_sql_fragment(
            expr,
            &mut |name| compose::tag_shadowed(name, self),
            self.functions.imported_sql_tags(),
        )
    }
}

fn collect_return_operand_traces(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    out: &mut Vec<CallSqlFragment>,
) {
    match unwrap_ts_wrappers(expr) {
        Expression::BinaryExpression(bin) if bin.operator == BinaryOperator::Addition => {
            collect_return_operand_traces(visitor, &bin.left, out);
            collect_return_operand_traces(visitor, &bin.right, out);
        }
        Expression::Identifier(id) => {
            if let Some(scope_index) = visitor.lookup_scope(id.name.as_str()) {
                if let Some(trace) = visitor
                    .call_fragment_state
                    .builders
                    .get(&(scope_index, id.name.to_string()))
                {
                    out.extend(trace.fragments.iter().cloned());
                }
            }
        }
        _ => {}
    }
}
