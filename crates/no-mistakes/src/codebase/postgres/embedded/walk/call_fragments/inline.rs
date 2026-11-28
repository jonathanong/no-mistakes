use super::super::calls::chain;
use super::super::ScopeVisitor;
use super::partial;
use crate::codebase::postgres::embedded::{first_call_argument, CallSqlFragment};
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
        let Some(scope_index) = self.lookup_scope(id.name.as_str()) else {
            return;
        };
        let key = (scope_index, id.name.to_string());
        match assign.operator {
            AssignmentOperator::Addition => {
                self.append_assignment_expr(key, &assign.right, assign.span.start);
            }
            AssignmentOperator::Assign => {
                if let Some(parts) = partial::self_addition_parts(&assign.right, id.name.as_str()) {
                    for part in parts {
                        self.append_assignment_expr(key.clone(), part, assign.span.start);
                    }
                } else {
                    self.clear_builder_fragment(scope_index, id.name.as_str());
                    let line = byte_offset_to_line(self.source, assign.span.start as usize);
                    self.note_builder_init(
                        scope_index,
                        id.name.as_str(),
                        &assign.right,
                        None,
                        line,
                    );
                }
            }
            _ => self.clear_builder_fragment(scope_index, id.name.as_str()),
        }
    }

    fn append_assignment_expr(&mut self, key: (usize, String), rhs: &Expression<'_>, site: u32) {
        let (context, alt_prefixes) = self
            .call_fragment_state
            .builders
            .get(&key)
            .map(|t| (t.prefix.clone(), t.alt_prefixes.clone()))
            .unwrap_or_default();
        let straight_line = self.control_depth == 0 && self.loop_depth == 0;
        if matches!(unwrap_ts_wrappers(rhs), Expression::Identifier(_))
            && partial::try_relation_append(
                self,
                &key,
                context.as_deref(),
                rhs,
                straight_line,
                site,
            )
        {
            return;
        }
        let added = expand_with_context(self, rhs, context.as_deref(), &alt_prefixes, site);
        if added.is_empty() {
            return;
        }
        let trace = self.call_fragment_state.builders.entry(key).or_default();
        partial::extend_prefix(trace, &added, straight_line);
        trace.fragments.extend(added);
    }
}

pub(super) fn expand_with_context(
    visitor: &ScopeVisitor<'_>,
    arg: &Expression<'_>,
    context: Option<&str>,
    alt_prefixes: &[String],
    site: u32,
) -> Vec<CallSqlFragment> {
    if alt_prefixes.is_empty() {
        partial::expr_fragments(visitor, arg, context, Some(site))
    } else {
        alt_prefixes
            .iter()
            .flat_map(|ctx| partial::expr_fragments(visitor, arg, Some(ctx), Some(site)))
            .collect()
    }
}

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
    let mut context = trace.and_then(|t| t.prefix.clone()).or_else(|| {
        visitor
            .scopes
            .get(scope_index)
            .and_then(|s| s.get(&key.1))
            .and_then(|b| b.sql.clone())
    });
    let alt_prefixes = trace.map(|t| t.alt_prefixes.clone()).unwrap_or_default();
    for link in links {
        if covered_sites.contains(&link.span.start) {
            continue;
        }
        let Some(arg) = first_call_argument(link) else {
            continue;
        };
        let added = expand_with_context(
            visitor,
            arg,
            context.as_deref(),
            &alt_prefixes,
            link.span.start,
        );
        if added.len() == 1 {
            if let Some(prefix) = &mut context {
                prefix.push_str(&added[0].sql);
            }
        }
        out.extend(added);
    }
    Some(out)
}

fn append_root_and_links<'a>(
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
