//! Source facts the conflict-ordering rule needs to normalize `ORDER BY` keys: which select
//! expressions are bound values, and which relations a bare column name can resolve to.
use super::pinned::expr_is_constant;
use super::single_row::projection_expr;
use sqlparser::ast::{Query, SetExpr, TableFactor, TableWithJoins};
use std::collections::BTreeSet;

/// A relation in the source `FROM` clause; `columns` is `None` when its columns are unknown
/// (a plain table, whose columns only the database knows).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlSourceRelation {
    pub qualifier: Option<String>,
    pub columns: Option<Vec<String>>,
}

/// Select-list expressions that are literals or bound parameters. Computed with the recovered
/// placeholder positions, so a user identifier spelled like a placeholder is not included. A
/// spelling shared with a non-constant projection is left out: the text cannot tell them apart.
pub(super) fn constant_projections(query: &Query, binds: &[(u32, u32)]) -> BTreeSet<String> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return BTreeSet::new();
    };
    let (constant, varying): (Vec<_>, Vec<_>) = select
        .projection
        .iter()
        .filter_map(projection_expr)
        .partition(|expr| expr_is_constant(expr, binds));
    let varying: BTreeSet<String> = varying.iter().map(ToString::to_string).collect();
    constant
        .iter()
        .map(ToString::to_string)
        .filter(|text| !varying.contains(text))
        .collect()
}

/// Every relation of a plain `SELECT ... FROM`, or `None` when the scope cannot be listed
/// (nested joins, set operations, pivots), in which case a bare name stays unresolved.
pub(super) fn source_relations(query: &Query) -> Option<Vec<SqlSourceRelation>> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return None;
    };
    let mut relations = Vec::new();
    for from in &select.from {
        push_relations(from, &mut relations)?;
    }
    Some(relations)
}

fn push_relations(from: &TableWithJoins, relations: &mut Vec<SqlSourceRelation>) -> Option<()> {
    relations.push(relation(&from.relation)?);
    for join in &from.joins {
        relations.push(relation(&join.relation)?);
    }
    Some(())
}

fn relation(factor: &TableFactor) -> Option<SqlSourceRelation> {
    let (alias, name) = match factor {
        TableFactor::Table { name, alias, .. } => {
            (alias, name.0.last().map(|part| part.to_string()))
        }
        TableFactor::Derived { alias, .. }
        | TableFactor::Function { alias, .. }
        | TableFactor::TableFunction { alias, .. }
        | TableFactor::UNNEST { alias, .. } => (alias, None),
        _ => return None,
    };
    let qualifier = alias
        .as_ref()
        .map(|alias| alias.name.value.clone())
        .or(name.map(|name| name.trim_matches('"').to_owned()))
        .map(|name| name.to_ascii_lowercase());
    let columns = alias
        .as_ref()
        .filter(|alias| !alias.columns.is_empty())
        .map(|alias| {
            alias
                .columns
                .iter()
                .map(|column| column.name.value.to_ascii_lowercase())
                .collect()
        });
    Some(SqlSourceRelation { qualifier, columns })
}
