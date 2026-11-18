use super::expression::compatible;
use super::{expression, Arm, Node, Recovered, ScopeVisitor, MAX_EMBEDDED_SQL_VARIANTS};
use crate::codebase::postgres::embedded::walk::variants::ValueKind;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BinaryOperator, Expression, LogicalOperator, UnaryOperator};

pub(super) fn recover(
    visitor: &ScopeVisitor<'_>,
    test: &Expression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    let child_depth = depth.checked_sub(1)?;
    match unwrap_ts_wrappers(test) {
        Expression::BooleanLiteral(value) => Some(vec![boolean(value.value, Vec::new())]),
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            let mut values = recover(visitor, &unary.argument, child_depth, constraints)?;
            for value in &mut values {
                value.value = ValueKind::Boolean(!value.truth());
            }
            Some(values)
        }
        Expression::BinaryExpression(binary)
            if matches!(
                binary.operator,
                BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
            ) =>
        {
            compare(
                strings(visitor, &binary.left, depth)?,
                strings(visitor, &binary.right, depth)?,
                binary.operator == BinaryOperator::StrictInequality,
                constraints,
            )
        }
        Expression::LogicalExpression(logical) if logical.operator != LogicalOperator::Coalesce => {
            let take_right = logical.operator == LogicalOperator::And;
            let mut result = Vec::new();
            for left in logical_operand(visitor, &logical.left, child_depth, constraints)? {
                if left.truth() == take_right {
                    let path = merge_choices(constraints, &left.choices);
                    for mut right in logical_operand(visitor, &logical.right, child_depth, &path)? {
                        right.choices = merge_choices(&left.choices, &right.choices);
                        push(&mut result, right)?;
                    }
                } else {
                    push(&mut result, left)?;
                }
            }
            Some(result)
        }
        _ => None,
    }
}

type StringChoice = (String, Vec<(u64, u32)>);

fn strings(
    visitor: &ScopeVisitor<'_>,
    operand: &Expression<'_>,
    depth: u8,
) -> Option<Vec<StringChoice>> {
    let operand = unwrap_ts_wrappers(operand);
    if !matches!(
        operand,
        Expression::Identifier(_)
            | Expression::StringLiteral(_)
            | Expression::StaticMemberExpression(_)
    ) {
        return None;
    }
    expression(visitor, operand, depth)?
        .into_iter()
        .map(|arm| match arm {
            Arm {
                node: Node::Text(text),
                choices,
                root: None,
                provenance: None,
                type_annotation,
            } if choices.is_empty() || type_annotation => Some((text, choices)),
            _ => None,
        })
        .collect()
}

fn compare(
    left: Vec<StringChoice>,
    right: Vec<StringChoice>,
    negate: bool,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    let mut result = Vec::new();
    for (left_text, left_choices) in left
        .iter()
        .filter(|(_, choices)| compatible(choices, constraints))
    {
        for (right_text, right_choices) in &right {
            if !compatible(right_choices, constraints) || !compatible(left_choices, right_choices) {
                continue;
            }
            let choices = merge_choices(left_choices, right_choices);
            let truth = (left_text == right_text) != negate;
            push(&mut result, boolean(truth, choices))?;
        }
    }
    Some(result)
}

fn boolean(truth: bool, choices: Vec<(u64, u32)>) -> Recovered {
    let mut value = Recovered::empty(1);
    value.value = ValueKind::Boolean(truth);
    value.choices = choices;
    value
}

fn merge_choices(left: &[(u64, u32)], right: &[(u64, u32)]) -> Vec<(u64, u32)> {
    let mut choices = left.to_vec();
    choices.extend_from_slice(right);
    choices.sort_unstable();
    choices.dedup();
    choices
}

fn push(values: &mut Vec<Recovered>, value: Recovered) -> Option<()> {
    if !values.contains(&value) {
        values.push(value);
    }
    (values.len() <= MAX_EMBEDDED_SQL_VARIANTS).then_some(())
}

fn logical_operand(
    visitor: &ScopeVisitor<'_>,
    test: &Expression<'_>,
    depth: u8,
    constraints: &[(u64, u32)],
) -> Option<Vec<Recovered>> {
    if let Some(values) = recover(visitor, test, depth, constraints) {
        return Some(values);
    }
    if depth == 0
        || !matches!(
            unwrap_ts_wrappers(test),
            Expression::Identifier(_) | Expression::StaticMemberExpression(_)
        )
    {
        return None;
    }
    let id = u64::from(oxc_span::GetSpan::span(test).start) | (1 << 60);
    Some(
        [false, true]
            .into_iter()
            .enumerate()
            .filter_map(|(arm, truth)| {
                let choices = vec![(id, arm as u32)];
                compatible(&choices, constraints).then(|| boolean(truth, choices))
            })
            .collect(),
    )
}
