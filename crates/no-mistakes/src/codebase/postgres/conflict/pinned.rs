//! Pin evidence for single- or inner-joined-relation `INSERT ... SELECT` sources and the
//! constant-value test shared with the conflict-ordering rule.
use super::single_row::{projection_expr, single_valued};
use super::SqlConflictInsertFact;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use sqlparser::ast::{
    Expr, GroupByExpr, JoinConstraint, JoinOperator, ObjectName, ObjectNamePart, Query, SetExpr,
    TableFactor, TableWithJoins, With,
};

mod predicates;
pub(super) use predicates::expr_is_constant;
pub use predicates::expression_is_constant;
use predicates::{collect_pins, RelationSpec};

/// One or more inner-joined plain relations whose top-level `WHERE` / `ON` equalities may bound
/// every relation to at most one row via catalog unique keys.
pub(super) fn pinned_source(query: &Query, binds: &[(u32, u32)]) -> Option<SqlPinnedSource> {
    if query.with.is_some() {
        return None;
    }
    let SetExpr::Select(select) = query.body.as_ref() else {
        return None;
    };
    if select.from.is_empty()
        || has_grouping_sets(&select.group_by)
        || !select
            .projection
            .iter()
            .all(|item| projection_expr(item).is_some_and(|expr| single_valued(expr, true, binds)))
    {
        return None;
    }
    let mut specs = Vec::new();
    let mut predicates = Vec::new();
    for from in &select.from {
        collect_from(from, &mut specs, &mut predicates)?;
    }
    if let Some(selection) = select.selection.as_ref() {
        predicates.push(selection);
    }
    let mut relations: Vec<SqlPinnedRelation> = specs
        .iter()
        .map(|spec| SqlPinnedRelation {
            table: spec.table.clone(),
            columns: Vec::new(),
        })
        .collect();
    let mut join_equalities = Vec::new();
    for predicate in predicates {
        collect_pins(
            predicate,
            &specs,
            binds,
            &mut relations,
            &mut join_equalities,
        );
    }
    let has_constant_pin = relations
        .iter()
        .any(|relation| !relation.columns.is_empty());
    let all_referenced = has_constant_pin
        && relations.iter().enumerate().all(|(index, relation)| {
            !relation.columns.is_empty()
                || join_equalities
                    .iter()
                    .any(|eq| eq.left_relation == index || eq.right_relation == index)
        });
    all_referenced.then_some(SqlPinnedSource {
        relations,
        join_equalities,
    })
}

fn has_grouping_sets(group_by: &GroupByExpr) -> bool {
    matches!(
        group_by,
        GroupByExpr::Expressions(exprs, _)
            if exprs
                .iter()
                .any(|expr| matches!(expr, Expr::GroupingSets(_) | Expr::Cube(_) | Expr::Rollup(_)))
    )
}

fn collect_from<'a>(
    from: &'a TableWithJoins,
    specs: &mut Vec<RelationSpec>,
    predicates: &mut Vec<&'a Expr>,
) -> Option<()> {
    specs.push(relation_spec(&from.relation)?);
    for join in &from.joins {
        match &join.join_operator {
            JoinOperator::Join(JoinConstraint::On(on))
            | JoinOperator::Inner(JoinConstraint::On(on)) => predicates.push(on),
            JoinOperator::CrossJoin(JoinConstraint::None) => {}
            _ => return None,
        }
        specs.push(relation_spec(&join.relation)?);
    }
    Some(())
}

fn relation_spec(factor: &TableFactor) -> Option<RelationSpec> {
    let TableFactor::Table {
        name,
        alias,
        args: None,
        ..
    } = factor
    else {
        return None;
    };
    if alias
        .as_ref()
        .is_some_and(|alias| !alias.columns.is_empty())
    {
        return None;
    }
    let qualifier = match alias {
        Some(alias) => ident_key(&alias.name),
        None => ident_key(object_name_ident(name)?),
    };
    let qualified_table = alias.is_none().then(|| two_part_name(name)).flatten();
    Some(RelationSpec {
        table: name.to_string(),
        qualifier,
        qualified_table,
    })
}

fn two_part_name(name: &ObjectName) -> Option<(String, String)> {
    match name.0.as_slice() {
        [ObjectNamePart::Identifier(schema), ObjectNamePart::Identifier(table)] => {
            Some((ident_key(schema), ident_key(table)))
        }
        _ => None,
    }
}

/// A plain relation and its columns equated to constants in top-level `WHERE` / `ON` conjuncts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlPinnedRelation {
    pub table: String,
    pub columns: Vec<String>,
}

/// An equality between columns of two distinct relations in a pinned source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlPinnedJoinEquality {
    pub left_relation: usize,
    pub left_column: String,
    pub right_relation: usize,
    pub right_column: String,
}

/// One or more inner-joined plain relations with constant pins and column join equalities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlPinnedSource {
    pub relations: Vec<SqlPinnedRelation>,
    pub join_equalities: Vec<SqlPinnedJoinEquality>,
}

/// Drop pin evidence for inserts where any pinned relation is named like a CTE in scope.
pub(super) fn forget_shadowed(inserts: &mut [SqlConflictInsertFact], with: &With) {
    for insert in inserts {
        let shadowed = insert.source.pinned_source.as_ref().is_some_and(|source| {
            source.relations.iter().any(|pinned| {
                let name = pinned
                    .table
                    .rsplit('.')
                    .next()
                    .unwrap_or_default()
                    .trim_matches('"');
                with.cte_tables
                    .iter()
                    .any(|cte| cte.alias.name.value.eq_ignore_ascii_case(name))
            })
        });
        if shadowed {
            insert.source.pinned_source = None;
        }
    }
}
