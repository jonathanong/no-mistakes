use super::super::resolve::compose;
use super::super::ScopeVisitor;
use super::{extend_prefix, partial};
use crate::codebase::postgres::embedded::{
    first_call_argument, tags, CallSqlFragment, EmbeddedSqlCall,
};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use crate::fx::FxHashSet;
use oxc_ast::ast::{
    AssignmentExpression, AssignmentOperator, AssignmentTarget, CallExpression, Expression,
};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn note_assignment_fragment(
        &mut self,
        assign: &AssignmentExpression<'_>,
    ) {
        if self.head_calls_only {
            return;
        }
        let AssignmentTarget::AssignmentTargetIdentifier(id) = &assign.left else {
            return;
        };
        let Some(scope_index) = self
            .scopes
            .iter()
            .rposition(|scope| scope.contains_key(id.name.as_str()))
        else {
            return;
        };
        let key = (scope_index, id.name.to_string());
        match assign.operator {
            AssignmentOperator::Addition => {
                let context = self
                    .call_fragment_state
                    .builders
                    .get(&key)
                    .and_then(|trace| trace.prefix.clone());
                let added = partial::expr_fragments(
                    self,
                    &assign.right,
                    context.as_deref(),
                    Some(assign.span.start),
                );
                if added.is_empty() {
                    return;
                }
                let trace = self.call_fragment_state.builders.entry(key).or_default();
                extend_prefix(
                    trace,
                    &added,
                    self.control_depth == 0 && self.loop_depth == 0,
                );
                trace.fragments.extend(added);
            }
            AssignmentOperator::Assign => {
                self.call_fragment_state.builders.remove(&key);
                let line = byte_offset_to_line(self.source, assign.span.start as usize);
                self.note_builder_init(scope_index, id.name.as_str(), &assign.right, None, line);
            }
            _ => {
                self.call_fragment_state.builders.remove(&key);
            }
        }
    }

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
        let Expression::Identifier(id) = unwrap_ts_wrappers(expr) else {
            return;
        };
        let Some(scope_index) = self
            .scopes
            .iter()
            .rposition(|scope| scope.contains_key(id.name.as_str()))
        else {
            return;
        };
        if let Some(trace) = self
            .call_fragment_state
            .builders
            .get(&(scope_index, id.name.to_string()))
        {
            if !trace.fragments.is_empty() {
                self.call_fragment_state
                    .function_returns
                    .entry(owner)
                    .or_default()
                    .extend(trace.fragments.iter().cloned());
            }
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
                if let Some(scope_index) = self
                    .scopes
                    .iter()
                    .rposition(|scope| scope.contains_key(id.name.as_str()))
                {
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
            Expression::CallExpression(inner)
                if matches!(unwrap_ts_wrappers(&inner.callee), Expression::Identifier(_)) =>
            {
                let Expression::Identifier(callee) = unwrap_ts_wrappers(&inner.callee) else {
                    return Vec::new();
                };
                let key = self.plain_key(callee.name.as_str());
                self.call_fragment_state
                    .function_returns
                    .get(&key)
                    .cloned()
                    .unwrap_or_default()
            }
            _ if recovered.variants.is_empty() => partial::expr_fragments(self, arg, None, None),
            _ => Vec::new(),
        }
    }

    pub(super) fn is_tagged_sql_fragment(&self, expr: &Expression<'_>) -> bool {
        tags::fragments::is_sql_fragment(
            expr,
            &mut |name| compose::tag_shadowed(name, self),
            self.functions.imported_sql_tags(),
        )
    }
}
