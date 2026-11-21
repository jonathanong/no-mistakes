use super::super::{Kind, Value};
use super::Evaluator;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, LogicalOperator};
use oxc_span::GetSpan;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[derive(Clone)]
enum Branch {
    Object(BTreeMap<String, Value>, PathBuf, usize),
    Absent,
    // `||` / `??` whose left side is not a known object. Keys stay unknown.
    Opaque,
}

impl Evaluator<'_, '_> {
    pub(super) fn folded_spread(
        &mut self,
        expression: &Expression<'_>,
        existing: &BTreeMap<String, Value>,
        incoming_opaque: bool,
    ) -> Option<Value> {
        if !is_branch(expression) {
            return None;
        }
        let branches = self.branches(expression)?;
        Some(self.merge_branches(
            branches,
            existing.get("exclude"),
            incoming_opaque,
            self.line(expression.span().start),
        ))
    }

    fn branches(&mut self, expression: &Expression<'_>) -> Option<Vec<Branch>> {
        match unwrap_ts_wrappers(expression) {
            Expression::ConditionalExpression(value) => {
                let mut branches = self.branches(&value.consequent)?;
                branches.extend(self.branches(&value.alternate)?);
                Some(branches)
            }
            Expression::LogicalExpression(value) => {
                let right = self.branches(&value.right)?;
                Some(combine(value.operator, self.branches(&value.left), right))
            }
            other => self.literal_branch(other),
        }
    }

    fn literal_branch(&mut self, expression: &Expression<'_>) -> Option<Vec<Branch>> {
        let value = self.eval(expression);
        match value.kind {
            Kind::Object(fields, false) => {
                Some(vec![Branch::Object(fields, value.file, value.line)])
            }
            Kind::Literal(serde_json::Value::Null) => Some(vec![Branch::Absent]),
            _ => None,
        }
    }

    fn merge_branches(
        &self,
        branches: Vec<Branch>,
        previous_exclude: Option<&Value>,
        incoming_opaque: bool,
        line: usize,
    ) -> Value {
        let mut absent = false;
        let mut opaque = false;
        let mut objects = Vec::new();
        for branch in branches {
            match branch {
                Branch::Absent => absent = true,
                Branch::Opaque => opaque = true,
                Branch::Object(fields, file, line) => objects.push((fields, file, line)),
            }
        }
        let mut properties = BTreeMap::new();
        if !objects.is_empty() {
            let mut keys = BTreeSet::new();
            for (fields, _, _) in &objects {
                keys.extend(fields.keys().cloned());
            }
            for key in keys {
                let value = merged_value(
                    &objects,
                    &key,
                    absent,
                    opaque,
                    previous_exclude,
                    incoming_opaque,
                );
                properties.insert(key, value);
            }
        }
        Value {
            kind: Kind::Object(properties, false),
            file: self.path.to_path_buf(),
            line,
        }
    }
}

fn is_branch(expression: &Expression<'_>) -> bool {
    matches!(
        unwrap_ts_wrappers(expression),
        Expression::ConditionalExpression(_) | Expression::LogicalExpression(_)
    )
}

fn combine(
    operator: LogicalOperator,
    left: Option<Vec<Branch>>,
    mut right: Vec<Branch>,
) -> Vec<Branch> {
    let Some(left) = left else {
        return match operator {
            LogicalOperator::And => {
                right.push(Branch::Absent);
                right
            }
            LogicalOperator::Or | LogicalOperator::Coalesce => {
                right.push(Branch::Opaque);
                right
            }
        };
    };
    if left.iter().all(|branch| matches!(branch, Branch::Absent)) {
        return match operator {
            LogicalOperator::And => vec![Branch::Absent],
            LogicalOperator::Or | LogicalOperator::Coalesce => right,
        };
    }
    let mut merged = Vec::new();
    for branch in left {
        match (&operator, branch) {
            (LogicalOperator::And, Branch::Object(..)) => merged.extend(right.clone()),
            (LogicalOperator::And, Branch::Absent) => merged.push(Branch::Absent),
            (LogicalOperator::And, Branch::Opaque) => merged.push(Branch::Opaque),
            (
                LogicalOperator::Or | LogicalOperator::Coalesce,
                Branch::Object(fields, file, line),
            ) => merged.push(Branch::Object(fields, file, line)),
            (LogicalOperator::Or | LogicalOperator::Coalesce, Branch::Absent) => {
                merged.extend(right.clone());
            }
            (LogicalOperator::Or | LogicalOperator::Coalesce, Branch::Opaque) => {
                merged.push(Branch::Opaque);
            }
        }
    }
    merged
}

fn merged_value(
    objects: &[(BTreeMap<String, Value>, PathBuf, usize)],
    key: &str,
    absent: bool,
    opaque: bool,
    previous_exclude: Option<&Value>,
    incoming_opaque: bool,
) -> Value {
    let present = objects
        .iter()
        .filter_map(|(fields, _, _)| fields.get(key))
        .collect::<Vec<_>>();
    let first = present[0];
    if key == "exclude" && !opaque {
        return super::exclude_paths::merged_exclude(
            present.into_iter().cloned().collect(),
            objects.len(),
            absent,
            previous_exclude,
            incoming_opaque,
        );
    }
    let same = present
        .iter()
        .all(|value| super::exclude_paths::same_kind(&value.kind, &first.kind));
    let blocked = absent || opaque;
    if !blocked && present.len() == objects.len() && same {
        return first.clone();
    }
    if !blocked && present.len() == objects.len() {
        let file = first.file.clone();
        let line = first.line;
        return Value {
            kind: Kind::OneOf(present.into_iter().cloned().collect()),
            file,
            line,
        };
    }
    super::exclude_paths::unknown_value(&first.file, first.line)
}
