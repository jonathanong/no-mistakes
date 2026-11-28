use super::super::resolve::compose;
use super::super::ScopeVisitor;
use super::partial;
use crate::codebase::postgres::embedded::{
    first_call_argument, placeholders, tags, CallSqlFragment, EmbeddedSqlCall,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{CallExpression, Expression};

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
        let fragments = if let Expression::Identifier(id) = unwrap_ts_wrappers(expr) {
            let Some(scope_index) = self.lookup_scope(id.name.as_str()) else {
                return;
            };
            self.call_fragment_state
                .builders
                .get(&(scope_index, id.name.to_string()))
                .map(|trace| trace.fragments.clone())
                .unwrap_or_default()
        } else if partial::is_sql_like_init(self, expr) {
            partial::expr_fragments(self, expr, None, None)
        } else {
            Vec::new()
        };
        if !fragments.is_empty() {
            self.call_fragment_state
                .function_returns
                .entry(owner)
                .or_default()
                .extend(fragments);
        }
    }

    pub(in crate::codebase::postgres::embedded::walk) fn collect_call_fragments(
        &self,
        call: &CallExpression<'_>,
        recovered: &EmbeddedSqlCall,
    ) -> Vec<CallSqlFragment> {
        if self.head_calls_only || !recovered.is_unanalyzable() {
            return Vec::new();
        }
        let covered_sites: FxHashSet<u32> = recovered
            .variants
            .iter()
            .flat_map(|variant| variant.append_sites.iter().copied())
            .collect();
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
                                let duplicate_init = fragment.append_site.is_none()
                                    && !recovered.variants.is_empty();
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
                        .cloned()
                        .unwrap_or_default();
                }
                if let Some(frags) = super::inline::inline_append_fragments(
                    self,
                    inner,
                    &covered_sites,
                    !recovered.variants.is_empty(),
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

pub(crate) fn publish_call_fragments(fragments: &mut [CallSqlFragment]) {
    for fragment in fragments {
        fragment.sql = placeholders::publish_placeholders(std::mem::take(&mut fragment.sql));
        if let Some(context) = fragment.context_sql.take() {
            fragment.context_sql = Some(placeholders::publish_placeholders(context));
        }
    }
}
