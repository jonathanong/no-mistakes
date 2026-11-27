use super::super::items::sql_name;
use super::{Columns, Relations};
use crate::idents::{ident_key, object_name_ident, object_name_key};
use crate::statements::limit::FixedCount;
use crate::statements::value::PlaceholderPositions;
use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, JoinOperator, ObjectName, TableFactor};
use std::collections::BTreeMap;

impl super::Relations {
    pub(super) fn begin_from(&mut self) {
        self.outer.clone_from(&self.names);
        self.outer_opaque = self.opaque;
    }

    pub(super) fn proof_parent(&self, lateral: bool) -> Self {
        if lateral {
            return self.clone();
        }
        let mut parent = self.clone();
        parent.names.clone_from(&self.outer);
        parent.opaque = self.outer_opaque;
        parent
    }
}

pub(super) fn relation_key(factor: &TableFactor) -> Option<String> {
    let (alias, name) = match factor {
        TableFactor::UNNEST { alias, .. } | TableFactor::Derived { alias, .. } => (alias, None),
        TableFactor::Function { alias, name, .. } => (alias, Some(object_name_key(name))),
        TableFactor::Table { alias, name, .. } => (alias, object_name_ident(name).map(ident_key)),
        _ => return None,
    };
    alias.as_ref().map(|alias| ident_key(&alias.name)).or(name)
}

pub(in crate::statements::bounds) fn null_extended_sides(operator: &JoinOperator) -> (bool, bool) {
    match operator {
        JoinOperator::Left(_) | JoinOperator::LeftOuter(_) => (false, true),
        JoinOperator::Right(_) | JoinOperator::RightOuter(_) => (true, false),
        JoinOperator::FullOuter(_) => (true, true),
        _ => (false, false),
    }
}

pub(super) fn table_columns(relations: &Relations, name: &ObjectName) -> Columns {
    let cte = (name.0.len() == 1)
        .then(|| object_name_ident(name).map(ident_key))
        .flatten();
    match cte.and_then(|cte| relations.outputs.get(&cte)) {
        Some(Some(map)) => Columns::Named(map.clone()),
        Some(None) => Columns::Any {
            table: None,
            null_extended: false,
        },
        None => Columns::Any {
            table: Some(sql_name(name)),
            null_extended: false,
        },
    }
}

pub(super) fn named_caller(
    alias: &Option<sqlparser::ast::TableAlias>,
    width: usize,
    proven: bool,
) -> Columns {
    Columns::Named(if proven {
        alias_names(alias)
            .into_iter()
            .take(width)
            .map(|name| (name, Some(FixedCount::Caller)))
            .collect()
    } else {
        BTreeMap::new()
    })
}

pub(super) fn function_caller(
    name: &ObjectName,
    args: &[FunctionArg],
    positions: PlaceholderPositions<'_>,
) -> bool {
    let builtin =
        name.0.len() == 1 && object_name_ident(name).is_some_and(|id| ident_key(id) == "unnest");
    builtin
        && args.iter().all(|arg| match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => caller_input(expr, positions),
            _ => false,
        })
}

pub(super) fn alias_names(alias: &Option<sqlparser::ast::TableAlias>) -> Vec<String> {
    alias.as_ref().map_or_else(Vec::new, |alias| {
        alias
            .columns
            .iter()
            .map(|column| ident_key(&column.name))
            .collect()
    })
}

pub(super) fn named_kind(
    names: &BTreeMap<String, Option<FixedCount>>,
    column: &str,
) -> Option<FixedCount> {
    match names.get(column) {
        Some(Some(kind)) => Some(*kind),
        Some(None) => Some(FixedCount::Nullable),
        None => None,
    }
}

pub(super) fn caller_input(expr: &Expr, positions: PlaceholderPositions<'_>) -> bool {
    match expr {
        Expr::Cast { expr, .. } | Expr::Nested(expr) => caller_input(expr, positions),
        Expr::Value(value) => matches!(value.value, sqlparser::ast::Value::Placeholder(_)),
        Expr::Identifier(ident) => {
            positions.is_some()
                && crate::statements::value::is_placeholder_ident_at(ident, positions)
        }
        _ => false,
    }
}
