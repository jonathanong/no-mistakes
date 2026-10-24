mod any_source;
mod array;
mod candidate;
mod correlated;
mod equality;
mod evidence;
mod expression;
mod query_array;
mod subquery;
use subquery::subquery_source;
mod resolver;
mod source_merge;

use super::Scope;
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundPin, SqlPinSource};
use resolver::Sourced;
use source_merge::merge;
use sqlparser::ast::{BinaryOperator, Expr, Query};

pub(super) use correlated::{projection_columns, Reads};
pub(super) use evidence::collect as collect_evidence;
pub(super) use resolver::Resolver;

/// Collect the pins that the conjuncts of `expr` impose on the `restricted` items.
pub(super) fn extract(
    expr: &Expr,
    resolver: &Resolver,
    restricted: &[usize],
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
    out: &mut Vec<(usize, SqlBoundPin)>,
) {
    let mut pin = |column: &Expr, sourced: Option<Sourced>, null_safe: bool| {
        if let (
            Some((item, column)),
            Some(Sourced {
                source,
                reads,
                qualified_reads,
            }),
        ) = (resolver.column(column), sourced)
        {
            if restricted.contains(&item) && resolver.is_table(item) {
                out.push((
                    item,
                    SqlBoundPin {
                        alternative: None,
                        column,
                        unqualified_candidate: false,
                        expression: None,
                        source,
                        null_safe,
                        reads,
                        qualified_reads,
                    },
                ));
            }
        }
    };
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: BinaryOperator::And,
            right,
        } => {
            extract(left, resolver, restricted, scope, positions, out);
            extract(right, resolver, restricted, scope, positions, out);
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::Eq,
            right,
        } => {
            equality::equate(left, right, false, resolver, &mut pin);
            candidate::equate(left, right, false, resolver, restricted, out);
            expression::equate(left, right, false, resolver, restricted, out);
        }
        Expr::IsNotDistinctFrom(left, right) => {
            equality::equate(left, right, true, resolver, &mut pin);
            candidate::equate(left, right, true, resolver, restricted, out);
            expression::equate(left, right, true, resolver, restricted, out);
        }
        // NULL selects one value only under NULLS NOT DISTINCT (or on an impossible NOT NULL
        // column). Reuse null-safe key evaluation so an ordinary nullable UNIQUE stays unproved.
        Expr::IsNull(column) => pin(
            column,
            Some(Sourced {
                source: SqlPinSource::Value,
                reads: Vec::new(),
                qualified_reads: Vec::new(),
            }),
            true,
        ),
        Expr::InList {
            expr,
            list,
            negated: false,
        } => {
            if let Some((item, _)) = resolver.column(expr) {
                let sources: Option<Vec<Sourced>> = list
                    .iter()
                    .map(|value| resolver.source(value, item))
                    .collect();
                pin(expr, sources.map(merge), false);
            }
            candidate::pin(expr, false, resolver, restricted, out, |item| {
                list.iter()
                    .map(|value| resolver.source(value, item))
                    .collect::<Option<Vec<_>>>()
                    .map(merge)
            });
        }
        Expr::InSubquery {
            expr,
            subquery,
            negated: false,
        } => {
            let source = subquery_source(subquery, resolver, scope, positions);
            if let Expr::Tuple(columns) = unwrap_expr(expr) {
                for column in columns {
                    pin(column, Some(source.clone()), false);
                }
            } else {
                pin(expr, Some(source.clone()), false);
            }
            candidate::pin(expr, false, resolver, restricted, out, |_| {
                Some(source.clone())
            });
        }
        Expr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } => {
            let resolved = resolver.column(left);
            if resolved.is_none() && resolver.candidate_column(left).is_none() {
                return;
            }
            let shared = resolver
                .projected_array(right)
                .or_else(|| query_array::source(right, resolver, scope, positions))
                .or_else(|| any_source::query(right, resolver, scope, positions));
            let source_for = |item| {
                shared
                    .clone()
                    .or_else(|| any_source::other(right, item, resolver, positions))
            };
            if let Some((item, _)) = resolved {
                pin(left, source_for(item), false);
            }
            candidate::pin(left, false, resolver, restricted, out, source_for);
        }
        // A bare boolean predicate is true only for one value. The catalog later decides
        // whether this column completes a primary or unique key; non-key columns get no credit.
        Expr::Identifier(_) | Expr::CompoundIdentifier(_) => {
            let source = Sourced {
                source: SqlPinSource::Value,
                reads: Vec::new(),
                qualified_reads: Vec::new(),
            };
            pin(expr, Some(source.clone()), false);
            candidate::pin(expr, false, resolver, restricted, out, |_| {
                Some(source.clone())
            });
        }
        _ => {}
    }
}

/// What a `LATERAL` source reads of the FROM items before it.
pub(super) fn reads_items(
    subquery: &Query,
    items: &[SqlBoundItem],
    scope: &Scope,
    positions: super::super::value::PlaceholderPositions<'_>,
) -> Reads {
    Resolver::new(items, scope.names(), positions).reads(subquery)
}
