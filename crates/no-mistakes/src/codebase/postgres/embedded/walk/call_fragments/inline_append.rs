use super::super::calls::chain;
use super::super::ScopeVisitor;
use super::partial::closed_relation_names;
use super::BuilderTrace;
use crate::codebase::postgres::embedded::helpers::sentinel;
use crate::codebase::postgres::embedded::{first_call_argument, placeholders, CallSqlFragment};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use crate::fx::FxHashSet;
use oxc_ast::ast::{CallExpression, Expression};

pub(super) fn inline_append_fragments(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    covered_sites: &FxHashSet<u32>,
    has_variants: bool,
) -> Option<Vec<CallSqlFragment>> {
    let (root, links) = append_root_and_links(call)?;
    let scope_index = visitor.lookup_scope(&root)?;
    let key = (scope_index, root);
    let trace = visitor.call_fragment_state.builders.get(&key);
    let mut out: Vec<CallSqlFragment> = trace
        .map(|t| {
            t.fragments
                .iter()
                .filter(|fragment| {
                    let covered = fragment
                        .append_site
                        .is_some_and(|site| covered_sites.contains(&site));
                    let duplicate_init = fragment.append_site.is_none() && has_variants;
                    !covered && !duplicate_init
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default();
    let mut state = BuilderTrace {
        prefix: trace.and_then(|t| t.prefix.clone()).or_else(|| {
            visitor
                .scopes
                .get(scope_index)
                .and_then(|s| s.get(&key.1))
                .and_then(|b| b.sql.clone())
        }),
        alt_prefixes: trace.map(|t| t.alt_prefixes.clone()).unwrap_or_default(),
        fragments: Vec::new(),
        decl_depth: (0, 0),
    };
    for link in links {
        if covered_sites.contains(&link.span.start) {
            continue;
        }
        let Some(arg) = first_call_argument(link) else {
            continue;
        };
        if let Some(ctx) = state
            .prefix
            .as_deref()
            .filter(|c| sentinel::in_relation_slot(c, c.len()))
        {
            if matches!(unwrap_ts_wrappers(arg), Expression::Identifier(_)) {
                if let Some(rels) = closed_relation_names(visitor, arg) {
                    let line = byte_offset_to_line(visitor.source, link.span.start as usize);
                    for rel in &rels {
                        out.push(CallSqlFragment {
                            sql: format!("{ctx}{rel}"),
                            line,
                            span: (link.span.start, link.span.start),
                            context_sql: None,
                            append_site: Some(link.span.start),
                        });
                    }
                    if rels.len() == 1 {
                        state.prefix = Some(format!("{ctx}{}", rels[0]));
                        state.alt_prefixes.clear();
                    } else {
                        state.alt_prefixes = rels.iter().map(|rel| format!("{ctx}{rel}")).collect();
                        state.prefix = None;
                    }
                    continue;
                }
            }
        }
        let added = super::inline::expand_with_context(
            visitor,
            arg,
            state.prefix.as_deref(),
            &state.alt_prefixes,
            link.span.start,
        );
        super::trace::extend_prefix(&mut state, &added, true);
        out.extend(added);
    }
    Some(out)
}

pub(super) fn append_root_and_links<'a>(
    call: &'a CallExpression<'a>,
) -> Option<(String, Vec<&'a CallExpression<'a>>)> {
    if let Some(fluent) = chain(call) {
        return Some((fluent.root, fluent.links));
    }
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.property.name != "append" || member.optional || call.optional {
        return None;
    }
    let Expression::Identifier(id) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    Some((id.name.to_string(), vec![call]))
}

pub(super) fn try_copy_builder_init(
    visitor: &mut ScopeVisitor<'_>,
    scope_index: usize,
    name: &str,
    init: &Expression<'_>,
) -> bool {
    let init = unwrap_ts_wrappers(init);
    if let Expression::Identifier(src) = init {
        if let Some(mut trace) = visitor
            .lookup_scope(src.name.as_str())
            .and_then(|s| {
                visitor
                    .call_fragment_state
                    .builders
                    .get(&(s, src.name.to_string()))
            })
            .cloned()
        {
            trace.decl_depth = (visitor.control_depth, visitor.loop_depth);
            visitor
                .call_fragment_state
                .builders
                .insert((scope_index, name.to_string()), trace);
            return true;
        }
    }
    if let Expression::CallExpression(call) = init {
        if let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) {
            let key = visitor.plain_key(callee.name.as_str());
            if let Some(mut returned) = visitor
                .call_fragment_state
                .function_returns
                .get(&key)
                .cloned()
            {
                returned.decl_depth = (visitor.control_depth, visitor.loop_depth);
                visitor
                    .call_fragment_state
                    .builders
                    .insert((scope_index, name.to_string()), returned);
                return true;
            }
        }
    }
    false
}

pub(super) fn call_has_sentinel(
    recovered: &crate::codebase::postgres::embedded::EmbeddedSqlCall,
) -> bool {
    recovered
        .sql_text
        .as_deref()
        .is_some_and(sentinel::contains)
        || recovered
            .variants
            .iter()
            .any(|v| sentinel::contains(&v.sql_text))
}

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn refresh_builder_alias(
        &mut self,
        name: &str,
        init: &Expression<'_>,
    ) {
        if self.head_calls_only {
            return;
        }
        if let Some(scope_index) = self.lookup_scope(name) {
            try_copy_builder_init(self, scope_index, name, init);
        }
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
