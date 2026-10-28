//! Structural reads and invalid use ranges share the original source walk.
use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::*;
use oxc_span::GetSpan;
#[derive(Clone, Copy)]
enum Read {
    Length,
    Index,
    At,
}
#[derive(Default)]
pub(in super::super) struct Facts {
    enabled: bool,
    reads: FxHashMap<Span, Read>,
    blocked: FxHashSet<Span>,
    pub tags: FxHashSet<Span>,
}
fn span(value: oxc_span::Span) -> Span {
    (value.start, value.end)
}
fn values<'a>(expression: &'a Expression<'a>) -> Option<&'a IdentifierReference<'a>> {
    let Expression::StaticMemberExpression(value) = unwrap_ts_wrappers(expression) else {
        return None;
    };
    if value.optional || value.property.name != "values" {
        return None;
    }
    let Expression::Identifier(root) = unwrap_ts_wrappers(&value.object) else {
        return None;
    };
    Some(root)
}
impl Facts {
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            ..Self::default()
        }
    }
    pub fn demanded(&self) -> bool {
        self.enabled && !self.reads.is_empty()
    }
    pub fn block(&mut self, value: oxc_span::Span) {
        if self.enabled {
            self.blocked.insert(span(value));
        }
    }
    pub fn member(&mut self, value: &StaticMemberExpression<'_>) {
        if !self.enabled || value.optional || value.property.name != "length" {
            return;
        }
        if let Some(root) = values(&value.object) {
            self.reads.insert(span(root.span), Read::Length);
        }
    }
    pub fn index(&mut self, value: &ComputedMemberExpression<'_>) {
        if !self.enabled || value.optional {
            return;
        }
        if let Some(root) = values(&value.object) {
            self.reads.insert(span(root.span), Read::Index);
        }
    }
    pub fn call(&mut self, value: &CallExpression<'_>) {
        if !self.enabled {
            return;
        }
        if !value.optional
            && value.arguments.len() == 1
            && value.arguments[0].as_expression().is_some()
        {
            if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&value.callee) {
                if !member.optional && member.property.name == "at" {
                    if let Some(root) = values(&member.object) {
                        self.reads.insert(span(root.span), Read::At);
                        return;
                    }
                }
            }
        }
        // A read used as another callee is outside this observation grammar.
        self.block(value.callee.span());
    }
    pub fn tag(&mut self, value: &TaggedTemplateExpression<'_>) {
        if !self.enabled {
            return;
        }
        if let Expression::Identifier(tag) = unwrap_ts_wrappers(&value.tag) {
            self.tags.insert(span(tag.span));
        }
        self.block(value.tag.span());
    }
    pub fn new_call(&mut self, value: &NewExpression<'_>) {
        self.block(value.callee.span());
    }
    pub fn target(&mut self, value: &SimpleAssignmentTarget<'_>) {
        self.block(value.span());
    }
    pub fn assignment(&mut self, value: &AssignmentExpression<'_>) {
        self.block(value.left.span());
    }
    pub fn unary(&mut self, value: &UnaryExpression<'_>) {
        if value.operator == UnaryOperator::Delete {
            self.block(value.argument.span());
        }
    }
    pub fn allowed(&self) -> FxHashSet<Span> {
        let mut blocked = self.blocked.iter().copied().collect::<Vec<_>>();
        blocked.sort_unstable();
        let mut end = 0;
        for value in &mut blocked {
            end = end.max(value.1);
            value.1 = end;
        }
        self.reads
            .keys()
            .copied()
            .filter(|reference| {
                let index = blocked.partition_point(|value| value.0 <= reference.0);
                index == 0 || blocked[index - 1].1 < reference.1
            })
            .collect()
    }
}
