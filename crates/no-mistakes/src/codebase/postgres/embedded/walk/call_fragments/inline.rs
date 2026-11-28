use super::super::ScopeVisitor;
use super::trace::extend_prefix;
use super::{partial, BuilderTrace};
use crate::codebase::postgres::embedded::CallSqlFragment;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{
    AssignmentExpression, AssignmentOperator, AssignmentTarget, BinaryOperator, Expression,
};

impl ScopeVisitor<'_> {
    pub(in crate::codebase::postgres::embedded::walk) fn clear_builder_fragment(
        &mut self,
        scope_index: usize,
        name: &str,
    ) {
        self.call_fragment_state
            .builders
            .remove(&(scope_index, name.to_string()));
        let key = self.plain_key(name);
        self.call_fragment_state.function_returns.remove(&key);
    }

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
            AssignmentOperator::LogicalOr
            | AssignmentOperator::LogicalAnd
            | AssignmentOperator::LogicalNullish => {
                let alt = partial::expr_fragments(self, &assign.right, None, None);
                if !alt.is_empty() {
                    let mut rhs = BuilderTrace::from_fragments(alt);
                    rhs.decl_depth = (self.control_depth, self.loop_depth);
                    self.call_fragment_state
                        .builders
                        .entry(key)
                        .or_default()
                        .merge(rhs);
                }
            }
            AssignmentOperator::Assign => {
                if let Some(parts) = self_addition_parts(&assign.right, id.name.as_str()) {
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
        let (context, alt_prefixes, decl_depth) = self
            .call_fragment_state
            .builders
            .get(&key)
            .map(|t| (t.prefix.clone(), t.alt_prefixes.clone(), t.decl_depth))
            .unwrap_or_default();
        let straight_line = (self.control_depth, self.loop_depth) == decl_depth;
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
        extend_prefix(trace, &added, straight_line);
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

fn self_addition_parts<'a>(
    expr: &'a Expression<'a>,
    target: &str,
) -> Option<Vec<&'a Expression<'a>>> {
    let mut parts = Vec::new();
    let mut cur = expr;
    while let Expression::BinaryExpression(bin) = unwrap_ts_wrappers(cur) {
        if bin.operator != BinaryOperator::Addition {
            return None;
        }
        parts.push(&bin.right);
        cur = &bin.left;
    }
    matches!(unwrap_ts_wrappers(cur), Expression::Identifier(id) if id.name == target).then(|| {
        parts.reverse();
        parts
    })
}
