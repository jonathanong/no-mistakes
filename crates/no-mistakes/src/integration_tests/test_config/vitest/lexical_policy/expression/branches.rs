use super::super::{Kind, Value};
use super::Evaluator;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{Expression, LogicalOperator};
use oxc_span::GetSpan;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone)]
enum Branch {
    Object(BTreeMap<String, Value>, PathBuf, usize),
    Absent,
}

impl Evaluator<'_, '_> {
    pub(super) fn folded_spread(&mut self, expression: &Expression<'_>) -> Option<Value> {
        if !is_branch(expression) {
            return None;
        }
        let branches = self.branches(expression)?;
        Some(self.merge_branches(branches, self.line(expression.span().start)))
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

    fn merge_branches(&self, branches: Vec<Branch>, line: usize) -> Value {
        let mut absent = false;
        let mut objects = Vec::new();
        for branch in branches {
            match branch {
                Branch::Absent => absent = true,
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
                let value = merged_value(&objects, &key, absent);
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
        right.push(Branch::Absent);
        return right;
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
            (
                LogicalOperator::Or | LogicalOperator::Coalesce,
                Branch::Object(fields, file, line),
            ) => merged.push(Branch::Object(fields, file, line)),
            (LogicalOperator::Or | LogicalOperator::Coalesce, Branch::Absent) => {
                merged.extend(right.clone());
            }
        }
    }
    merged
}

fn merged_value(
    objects: &[(BTreeMap<String, Value>, PathBuf, usize)],
    key: &str,
    absent: bool,
) -> Value {
    let present = objects
        .iter()
        .filter_map(|(fields, _, _)| fields.get(key))
        .collect::<Vec<_>>();
    let first = present[0];
    let same = present
        .iter()
        .all(|value| same_kind(&value.kind, &first.kind));
    if !absent && present.len() == objects.len() && same {
        return first.clone();
    }
    if !absent && present.len() == objects.len() {
        let file = first.file.clone();
        let line = first.line;
        return Value {
            kind: Kind::OneOf(present.into_iter().cloned().collect()),
            file,
            line,
        };
    }
    unknown_value(&first.file, first.line)
}

fn unknown_value(file: &Path, line: usize) -> Value {
    Value {
        kind: Kind::Unknown,
        file: file.to_path_buf(),
        line,
    }
}

fn same_kind(left: &Kind, right: &Kind) -> bool {
    match (left, right) {
        (Kind::Literal(left), Kind::Literal(right)) => left == right,
        (Kind::Unknown, Kind::Unknown) => true,
        (Kind::Array(left), Kind::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| same_kind(&left.kind, &right.kind))
        }
        (Kind::Object(left, left_opaque), Kind::Object(right, right_opaque)) => {
            left_opaque == right_opaque
                && left.len() == right.len()
                && left.iter().zip(right).all(
                    |((left_key, left_value), (right_key, right_value))| {
                        left_key == right_key && same_kind(&left_value.kind, &right_value.kind)
                    },
                )
        }
        (Kind::OneOf(left), Kind::OneOf(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| same_kind(&left.kind, &right.kind))
        }
        _ => false,
    }
}
