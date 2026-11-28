use super::calls::chain;
use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{first_call_argument, CallSqlFragment};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::GetSpan;

mod collect;
mod inline;
mod partial;
pub(crate) use partial::publish_call_fragments;

const MAX_PREFIX_BYTES: usize = 4096;

#[derive(Clone, Default)]
pub(super) struct BuilderTrace {
    pub(super) prefix: Option<String>,
    pub(super) alt_prefixes: Vec<String>,
    pub(super) fragments: Vec<CallSqlFragment>,
}

#[derive(Default)]
pub(super) struct CallFragmentState {
    pub(super) builders: FxHashMap<(usize, String), BuilderTrace>,
    pub(super) function_returns: FxHashMap<(u32, String), Vec<CallSqlFragment>>,
    pub(super) call_fragments: FxHashMap<(u32, u32), Vec<CallSqlFragment>>,
    pub(super) param_appends: Vec<CallSqlFragment>,
}

impl ScopeVisitor<'_> {
    pub(super) fn clear_builder_fragment(&mut self, scope_index: usize, name: &str) {
        self.call_fragment_state
            .builders
            .remove(&(scope_index, name.to_string()));
        let key = self.plain_key(name);
        self.call_fragment_state.function_returns.remove(&key);
    }

    pub(super) fn clear_builder_fragment_by_name(&mut self, name: &str) {
        if let Some(index) = self.lookup_scope(name) {
            self.clear_builder_fragment(index, name);
        }
    }

    pub(super) fn note_builder_init(
        &mut self,
        scope_index: usize,
        name: &str,
        init: &Expression<'_>,
        sql: Option<&str>,
        line: u32,
    ) {
        if self.head_calls_only {
            return;
        }
        let init = unwrap_ts_wrappers(init);
        if let Expression::CallExpression(call) = init {
            if let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) {
                let key = self.plain_key(callee.name.as_str());
                if let Some(returned) = self.call_fragment_state.function_returns.get(&key).cloned()
                {
                    let prefix = returned.first().map(|item| item.sql.clone());
                    self.call_fragment_state.builders.insert(
                        (scope_index, name.to_string()),
                        BuilderTrace {
                            prefix,
                            alt_prefixes: Vec::new(),
                            fragments: returned,
                        },
                    );
                    return;
                }
            }
        }
        if sql.is_none() && !partial::is_sql_like_init(self, init) {
            return;
        }
        let span = init.span();
        let fragments = if let Some(text) = sql.filter(|text| !text.trim().is_empty()) {
            vec![CallSqlFragment {
                sql: text.to_string(),
                line,
                span: (span.start, span.end),
                context_sql: None,
                append_site: None,
            }]
        } else {
            partial::expr_fragments(self, init, None, None)
        };
        if fragments.is_empty() {
            return;
        }
        let prefix = fragments
            .first()
            .map(|item| item.sql.clone())
            .filter(|text| text.len() <= MAX_PREFIX_BYTES);
        self.call_fragment_state.builders.insert(
            (scope_index, name.to_string()),
            BuilderTrace {
                prefix,
                alt_prefixes: Vec::new(),
                fragments,
            },
        );
    }

    pub(super) fn note_call_fragment_append(&mut self, call: &CallExpression<'_>) {
        if self.head_calls_only || self.consumed_links.contains(&call.span.start) {
            return;
        }
        if let Some(fluent) = chain(call) {
            for link in &fluent.links {
                self.record_append_link(&fluent.root, link);
            }
            return;
        }
        let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
            return;
        };
        if member.property.name != "append" || member.optional || call.optional {
            return;
        }
        let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) else {
            return;
        };
        self.record_append_link(id.name.as_str(), call);
    }

    fn record_append_link(&mut self, receiver: &str, call: &CallExpression<'_>) {
        if call.arguments.len() != 1 {
            return;
        }
        let Some(arg) = first_call_argument(call) else {
            return;
        };
        let Some((scope_index, binding)) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(idx, scope)| scope.get(receiver).map(|b| (idx, b.clone())))
        else {
            return;
        };
        let key = (scope_index, receiver.to_string());
        let known_builder = binding.sql_builder
            || binding.builder_identity.is_some()
            || binding.sql.is_some()
            || self.call_fragment_state.builders.contains_key(&key)
            || self.is_tagged_sql_fragment(arg);
        if !known_builder {
            return;
        }
        let (context, alt_prefixes) = self
            .call_fragment_state
            .builders
            .get(&key)
            .map(|t| (t.prefix.clone(), t.alt_prefixes.clone()))
            .unwrap_or_default();
        let context = context.or_else(|| binding.sql.clone());
        let straight_line = self.control_depth == 0 && self.loop_depth == 0;
        if binding.initialized
            && partial::try_relation_append(
                self,
                &key,
                context.as_deref(),
                arg,
                straight_line,
                call.span.start,
            )
        {
            return;
        }
        let added = if alt_prefixes.is_empty() {
            partial::expr_fragments(self, arg, context.as_deref(), Some(call.span.start))
        } else {
            alt_prefixes
                .iter()
                .flat_map(|ctx| {
                    partial::expr_fragments(self, arg, Some(ctx), Some(call.span.start))
                })
                .collect()
        };
        if added.is_empty() {
            return;
        }
        if !binding.initialized {
            self.call_fragment_state.param_appends.extend(added);
            return;
        }
        let trace = self.call_fragment_state.builders.entry(key).or_default();
        if trace.prefix.is_none() {
            trace.prefix = context;
        }
        partial::extend_prefix(trace, &added, straight_line);
        trace.fragments.extend(added);
    }
}
