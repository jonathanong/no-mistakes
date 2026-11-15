use super::{variants::Recovered, ScopeVisitor};
use crate::codebase::postgres::embedded::MAX_EMBEDDED_SQL_VARIANTS;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use crate::fx::FxHashMap;
use oxc_ast::ast::*;
use oxc_span::GetSpan;
use std::collections::{BTreeMap, HashMap};
mod assertions;
#[cfg(test)]
mod controls_tests;
mod expression;
#[cfg(test)]
mod extra_escape_tests;
#[cfg(test)]
mod followup_tests;
#[cfg(test)]
mod member_ownership_tests;
mod members;
#[cfg(test)]
mod parent_fields_tests;
mod state;
use expression::expression;
#[cfg(test)]
mod tests;
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
    assertions: FxHashMap<String, crate::codebase::postgres::embedded::SqlIdentifierAssertion>,
}
impl ClosedValues {
    pub(super) fn push(&mut self) {
        self.frames.push(FxHashMap::default());
    }
    pub(super) fn pop(&mut self) {
        self.frames.pop();
    }
    pub(super) fn roots(
        &self,
        scopes: &[HashMap<String, super::BindingState>],
        name: &str,
    ) -> Vec<String> {
        let Some(i) = scopes.iter().rposition(|scope| scope.contains_key(name)) else {
            return Vec::new();
        };
        self.frames
            .get(i)
            .and_then(|frame| frame.get(name))
            .into_iter()
            .flatten()
            .filter(|arm| !matches!(arm.node, Node::Text(_)))
            .filter_map(|arm| arm.root.clone())
            .collect()
    }
    pub(super) fn has(&self, scopes: &[HashMap<String, super::BindingState>], name: &str) -> bool {
        scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, scope)| scope.contains_key(name))
            .is_some_and(|(i, _)| {
                self.frames
                    .get(i)
                    .is_some_and(|frame| frame.contains_key(name))
            })
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
pub(super) fn recover(visitor: &ScopeVisitor<'_>, expr: &Expression<'_>) -> Option<Vec<Recovered>> {
    if matches!(
        unwrap_ts_wrappers(expr),
        Expression::StringLiteral(_) | Expression::TemplateLiteral(_)
    ) {
        return None;
    }
    let arms = expression(visitor, expr, 8)?;
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
