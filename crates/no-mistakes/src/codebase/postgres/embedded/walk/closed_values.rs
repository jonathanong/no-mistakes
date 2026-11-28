use super::{variants::Recovered, ScopeVisitor};
use crate::codebase::postgres::embedded::MAX_EMBEDDED_SQL_VARIANTS;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use crate::fx::FxHashMap;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use std::collections::{BTreeMap, HashMap};

/// Helper-object mark. Type-annotation keys use bit 62, and `select` copies
/// those choices onto an indexed record, so this mark must not share that bit.
const HELPER_OBJECT_CHOICE: u64 = 1 << 61;

mod assertions;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod controls_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod correlation_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod edge_tests;
mod expression;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod extra_escape_tests;
#[cfg(test)]
mod fixture;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod followup_tests;
mod frozen_return;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod member_ownership_tests;
mod members;
mod namespace_types;
mod parameter_members;
#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod parent_fields_tests;
mod predicate;
mod relation;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod relation_append_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod relation_edge_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod relation_tests;
mod state;
use expression::expression;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod hasown_edge_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod hasown_tests;
#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
mod type_names;
mod types;

#[derive(Clone)]
enum Node {
    Text(String),
    Object(BTreeMap<String, Node>),
    Array(Vec<Node>),
}
#[derive(Clone)]
struct Arm {
    node: Node,
    choices: Vec<(u64, u32)>,
    root: Option<String>,
    provenance: Option<super::resolve::functions::provenance::RecoveredSql>,
    /// The text is a parameter's type annotation, not a runtime value.
    type_annotation: bool,
}
#[derive(Default)]
pub(super) struct ClosedValues {
    frames: Vec<FxHashMap<String, Vec<Arm>>>,
    relation_frames: Vec<FxHashMap<String, Vec<Arm>>>,
    const_literal_arrays: Vec<FxHashMap<String, Vec<String>>>,
    type_frames: Vec<FxHashMap<String, Vec<String>>>,
    relation_type_frames: Vec<FxHashMap<String, Vec<String>>>,
    namespace_shadows: FxHashMap<u32, std::sync::Arc<crate::fx::FxHashSet<String>>>,
    assertions: FxHashMap<String, crate::codebase::postgres::embedded::SqlIdentifierAssertion>,
}
impl ClosedValues {
    pub(in crate::codebase::postgres::embedded::walk) fn push(&mut self) {
        self.frames.push(FxHashMap::default());
        self.relation_frames.push(FxHashMap::default());
        self.const_literal_arrays.push(FxHashMap::default());
        self.type_frames.push(FxHashMap::default());
        self.relation_type_frames.push(FxHashMap::default());
    }
    pub(in crate::codebase::postgres::embedded::walk) fn pop(&mut self) {
        self.frames.pop();
        self.relation_frames.pop();
        self.const_literal_arrays.pop();
        self.type_frames.pop();
        self.relation_type_frames.pop();
    }
    pub(super) fn remove_at(&mut self, index: usize, name: &str) {
        if let Some(frame) = self.frames.get_mut(index) {
            frame.remove(name);
        }
        if let Some(frame) = self.relation_frames.get_mut(index) {
            frame.remove(name);
        }
        if let Some(frame) = self.const_literal_arrays.get_mut(index) {
            frame.remove(name);
        }
    }
}
fn lookup(visitor: &ScopeVisitor<'_>, name: &str) -> Option<Vec<Arm>> {
    let i = visitor
        .scopes
        .iter()
        .rposition(|scope| scope.contains_key(name))?;
    visitor
        .literal_objects
        .closed
        .frames
        .get(i)?
        .get(name)
        .cloned()
}
pub(super) use relation::recover_relation;

pub(super) fn recover(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>) -> Option<Vec<Recovered>> {
    if matches!(
        unwrap_ts_wrappers(expr),
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
    ) {
        return None;
    }
    let arms = expression(visitor, expr, 8)?;
    materialize_arms(visitor, expr, arms)
}

fn materialize_arms(
    visitor: &ScopeVisitor<'_>,
    expr: &Expression<'_>,
    arms: Vec<Arm>,
) -> Option<Vec<Recovered>> {
    if arms
        .iter()
        .all(|arm| arm.choices.is_empty() && arm.root.is_none())
    {
        return None;
    }
    let mut result = Vec::new();
    for arm in arms {
        let Node::Text(mut sql) = arm.node else {
            return None;
        };
        if sql.contains('\0')
            || crate::codebase::postgres::embedded::helpers::sentinel::contains(&sql)
        {
            return None;
        }
        if let Some(root) = arm.root {
            sql = visitor.closed_splice(root, sql)?;
        }
        let mut value = Recovered::empty(byte_offset_to_line(
            visitor.source,
            expr.span().start as usize,
        ));
        value.origins = vec![expr.span().start; sql.len()];
        value.sql = sql;
        if let Some(source) = arm
            .provenance
            .filter(|source| source.sql == value.sql && source.origins.len() == value.sql.len())
        {
            value.line = source.line;
            value.origins = source.origins;
            value.positions = source.positions;
            value.invocations = source.invocations;
        }
        if let Expression::CallExpression(call) = unwrap_ts_wrappers(expr) {
            value.invocations.record(value.sql.len(), call.span.start);
        }
        value.choices = arm.choices;
        value.type_annotation = arm.type_annotation;
        value.enumerated = true;
        value.closed_domain = true;
        result.push(value);
    }
    (result.len() <= MAX_EMBEDDED_SQL_VARIANTS && !result.is_empty()).then_some(result)
}

pub(super) fn closed_call(visitor: &ScopeVisitor<'_>, call: &CallExpression<'_>) -> bool {
    assertions::recover(visitor,call,8).is_some_and(|arms| !arms.is_empty() && arms.iter().all(|arm| matches!(&arm.node,Node::Text(text) if !text.contains('\0') && !crate::codebase::postgres::embedded::helpers::sentinel::contains(text))))
}

pub(super) fn recover_predicate(
    visitor: &ScopeVisitor<'_>,
    test: &Expression<'_>,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if visitor.head_calls_only || visitor.loop_depth > 0 {
        return None;
    }
    predicate::recover(visitor, test, 8, constraints)
}
