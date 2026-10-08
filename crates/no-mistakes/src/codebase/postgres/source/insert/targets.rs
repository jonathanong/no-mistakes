use super::parsing::{
    assignment::{Step, Target},
    expressions::Located,
};
use crate::codebase::postgres::source::{
    expressions::{expression, identifier, name},
    locations::Locations,
    types::*,
};

pub(super) fn project(target: &Target, locations: &Locations<'_>) -> PostgresSqlAssignmentTarget {
    let subscripts = target
        .subscripts
        .iter()
        .map(|index| located(index, locations))
        .collect::<Vec<_>>();
    let indirection = target
        .indirection
        .iter()
        .any(|step| matches!(step, Step::Field(_)))
        .then(|| {
            target
                .indirection
                .iter()
                .map(|step| match step {
                    Step::Subscript { index, span } => PostgresSqlAssignmentStep::Subscript {
                        expression: Box::new(subscripts[*index].clone()),
                        span: locations.span(*span),
                    },
                    Step::Field(field) => PostgresSqlAssignmentStep::Field {
                        name: identifier(field),
                        span: locations.span(field.span),
                    },
                })
                .collect()
        });
    PostgresSqlAssignmentTarget {
        base: expression(&target.base, locations),
        subscripts,
        indirection,
        span: locations.span(target.span),
    }
}

pub(super) fn located(value: &Located, locations: &Locations<'_>) -> PostgresSqlExpression {
    super::spans::expression(&value.expression, value.span, &value.delimiters, locations)
}

pub(super) fn value(
    expr: &sqlparser::ast::Expr,
    facts: Option<&super::parsing::assignment::Facts>,
    locations: &Locations<'_>,
) -> PostgresSqlExpression {
    match facts {
        Some(facts) => {
            super::spans::expression(expr, facts.value_span, &facts.delimiters, locations)
        }
        None => expression(expr, locations),
    }
}

pub(super) fn conflict(
    target: Option<&sqlparser::ast::ConflictTarget>,
    facts: Option<&super::parsing::ConflictFacts>,
    locations: &Locations<'_>,
) -> PostgresSqlConflictTarget {
    use sqlparser::ast::ConflictTarget;
    if let Some(facts) = facts.filter(|facts| !facts.expressions.is_empty()) {
        let operator_classes = facts
            .expressions
            .iter()
            .any(|arbiter| arbiter.operator_class.is_some())
            .then(|| {
                facts
                    .expressions
                    .iter()
                    .map(|arbiter| {
                        arbiter
                            .operator_class
                            .as_ref()
                            .map(
                                |(class, span, parameters)| PostgresSqlArbiterOperatorClass {
                                    name: name(class),
                                    parameters: parameters
                                        .iter()
                                        .map(|(name, value)| PostgresSqlArbiterParameter {
                                            name: identifier(name),
                                            value: located(value, locations),
                                        })
                                        .collect(),
                                    span: locations.span(*span),
                                },
                            )
                    })
                    .collect()
            });
        PostgresSqlConflictTarget::Expressions {
            expressions: facts
                .expressions
                .iter()
                .map(|arbiter| located(&arbiter.expression, locations))
                .collect(),
            operator_classes,
        }
    } else {
        match target {
            None => PostgresSqlConflictTarget::Omitted,
            Some(ConflictTarget::Columns(columns)) => PostgresSqlConflictTarget::Columns {
                columns: columns.iter().map(identifier).collect(),
            },
            Some(ConflictTarget::OnConstraint(constraint)) => {
                PostgresSqlConflictTarget::Constraint {
                    name: name(constraint),
                }
            }
        }
    }
}
