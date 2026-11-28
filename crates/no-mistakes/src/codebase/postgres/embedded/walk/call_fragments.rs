use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{
    first_call_argument, CallSqlFragment, MAX_EMBEDDED_SQL_VARIANTS,
};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashMap;
use oxc_ast::ast::{CallExpression, Expression};
use oxc_span::GetSpan;

mod collect;
mod inline;
mod inline_append;
mod partial;
pub(crate) use collect::publish_call_fragments;

const MAX_PREFIX_BYTES: usize = 4096;

#[derive(Clone, Default)]
pub(super) struct BuilderTrace {
    pub(super) prefix: Option<String>,
    pub(super) alt_prefixes: Vec<String>,
    pub(super) fragments: Vec<CallSqlFragment>,
    pub(super) decl_depth: (usize, usize),
}

impl BuilderTrace {
    pub(super) fn merge(&mut self, other: Self) {
        if self.fragments.is_empty() && self.prefix.is_none() && self.alt_prefixes.is_empty() {
            *self = other;
            return;
        }
        let mut prefixes = Vec::new();
        for p in self
            .prefix
            .take()
            .into_iter()
            .chain(self.alt_prefixes.drain(..))
            .chain(other.prefix)
            .chain(other.alt_prefixes)
        {
            if !prefixes.contains(&p) && prefixes.len() < MAX_EMBEDDED_SQL_VARIANTS {
                prefixes.push(p);
            }
        }
        if prefixes.len() == 1 {
            self.prefix = prefixes.pop();
        } else {
            self.alt_prefixes = prefixes;
        }
        self.fragments.extend(other.fragments);
    }
}

#[derive(Default)]
pub(super) struct CallFragmentState {
    pub(super) builders: FxHashMap<(usize, String), BuilderTrace>,
    pub(super) function_returns: FxHashMap<(u32, String), BuilderTrace>,
    pub(super) call_fragments: FxHashMap<(u32, u32), Vec<CallSqlFragment>>,
    pub(super) param_appends: Vec<CallSqlFragment>,
}

impl ScopeVisitor<'_> {
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
        if inline_append::try_copy_builder_init(self, scope_index, name, init) {
            return;
        }
        if sql.is_none() && !partial::is_sql_like_init(self, init) {
            return;
        }
        let span = init.span();
        let from_expr = partial::expr_fragments(self, init, None, None);
        let prefer_expr = from_expr.len() > 1
            || (!from_expr.is_empty()
                && (sql.is_none() || matches!(init, Expression::TemplateLiteral(_))));
        let fragments = if prefer_expr {
            from_expr
        } else if let Some(text) = sql.filter(|text| !text.trim().is_empty()) {
            vec![CallSqlFragment {
                sql: text.to_string(),
                line,
                span: (span.start, span.end),
                context_sql: None,
                append_site: None,
            }]
        } else {
            from_expr
        };
        if fragments.is_empty() {
            return;
        }
        let mut trace = BuilderTrace::from_fragments(fragments);
        trace.decl_depth = (self.control_depth, self.loop_depth);
        self.call_fragment_state
            .builders
            .insert((scope_index, name.to_string()), trace);
    }

    pub(super) fn note_call_fragment_append(&mut self, call: &CallExpression<'_>) {
        if self.head_calls_only || self.consumed_links.contains(&call.span.start) {
            return;
        }
        let Some((root, links)) = inline_append::append_root_and_links(call) else {
            return;
        };
        for link in links {
            self.record_append_link(&root, link);
        }
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
        let (context, alt_prefixes, decl_depth) = self
            .call_fragment_state
            .builders
            .get(&key)
            .map(|t| (t.prefix.clone(), t.alt_prefixes.clone(), t.decl_depth))
            .unwrap_or_default();
        let context = context.or_else(|| binding.sql.clone());
        let straight_line = (self.control_depth, self.loop_depth) == decl_depth;
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
        let added = inline::expand_with_context(
            self,
            arg,
            context.as_deref(),
            &alt_prefixes,
            call.span.start,
        );
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
        inline::extend_prefix(trace, &added, straight_line);
        trace.fragments.extend(added);
    }
}
