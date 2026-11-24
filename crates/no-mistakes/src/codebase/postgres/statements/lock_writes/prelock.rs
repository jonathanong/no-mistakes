use super::{
    helpers::{access, column, qualifier},
    row_locks, LockWriteFacts, PlaceholderPositions, RowAccess,
};
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{
    BinaryOperator, Expr, GroupByExpr, LockType, Query, SelectItem, SetExpr, Statement,
    TableFactor, TableInheritance, Update, Value,
};
mod expressions;
mod upsert;
use expressions::{plain_assignments, plain_value};
pub(super) fn collect<const ADVANCED: bool>(
    statement: &Statement,
    out: &mut LockWriteFacts,
    positions: PlaceholderPositions<'_>,
    redundant_prelocks: bool,
) {
    match statement {
        Statement::Query(query) if redundant_prelocks => {
            out.prelock_attempt = query
                .locks
                .iter()
                .any(|lock| lock.lock_type == LockType::Update && lock.nonblock.is_none());
            if out.prelock_attempt {
                out.prelock = locked_key(query, positions);
            }
        }
        Statement::Update(update) if redundant_prelocks => {
            out.conditional_write_keys = update_keys(update, positions).unwrap_or_default();
        }
        Statement::Insert(insert) if ADVANCED => {
            if let Some(keys) = upsert::upsert_keys(insert, positions) {
                if redundant_prelocks {
                    if keys.len() == 1 {
                        out.conditional_write_keys = keys.clone();
                        out.conditional_write_unique = true;
                    } else {
                        out.multi_row_upsert_keys = Some(keys.clone().into_boxed_slice());
                    }
                }
                out.upsert_write_keys.extend(keys);
            }
        }
        _ => {}
    }
}

fn locked_key(query: &Query, positions: PlaceholderPositions<'_>) -> Option<(RowAccess, bool)> {
    let SetExpr::Select(select) = query.body.as_ref() else {
        return None;
    };
    let [SelectItem::UnnamedExpr(projection)] = select.projection.as_slice() else {
        return None;
    };
    let [lock] = query.locks.as_slice() else {
        return None;
    };
    if lock.lock_type != LockType::Update
        || lock.nonblock.is_some()
        || query.with.is_some()
        || query.order_by.is_some()
        || query.limit_clause.is_some()
        || query.fetch.is_some()
        || select.distinct.is_some()
        || select.into.is_some()
        || select.having.is_some()
        || select.qualify.is_some()
        || !matches!(&select.group_by, GroupByExpr::Expressions(groups, _) if groups.is_empty())
        || !matches!(unwrap_expr(projection), Expr::Value(value) if matches!(&value.value, Value::Number(value, _) if value == "1"))
    {
        return None;
    }
    let [table] = select.from.as_slice() else {
        return None;
    };
    let only = plain_table(&table.relation)?;
    let qualifier = qualifier(&table.relation)?;
    equality(select.selection.as_ref()?, &qualifier, positions)?;
    let row = row_locks::collect_legacy(query, positions, true);
    row.relation.is_some().then_some((row, only))
}

fn update_keys(update: &Update, positions: PlaceholderPositions<'_>) -> Option<Vec<RowAccess>> {
    if !update.table.joins.is_empty()
        || update.from.is_some()
        || !plain_assignments(&update.assignments)
    {
        return None;
    }
    let target = &update.table.relation;
    plain_table(target)?;
    let relation = access(target, None, positions).relation?;
    let qualifier = qualifier(target)?;
    let mut keys = Vec::new();
    and_keys(update.selection.as_ref()?, &qualifier, positions, &mut keys)?;
    let first = &keys.first()?.column;
    if keys.iter().all(|key| &key.column == first) {
        return None;
    }
    Some(
        keys.into_iter()
            .map(|key| key.row(relation.clone()))
            .collect(),
    )
}

fn and_keys(
    predicate: &Expr,
    qualifier: &str,
    positions: PlaceholderPositions<'_>,
    keys: &mut Vec<Key>,
) -> Option<()> {
    if let Expr::BinaryOp {
        left,
        op: BinaryOperator::And,
        right,
    } = unwrap_expr(predicate)
    {
        and_keys(left, qualifier, positions, keys)?;
        and_keys(right, qualifier, positions, keys)
    } else {
        keys.push(equality(predicate, qualifier, positions)?);
        Some(())
    }
}

struct Key {
    column: String,
    value: String,
}

impl Key {
    fn row(self, relation: String) -> RowAccess {
        RowAccess {
            relation: Some(relation),
            key: Some((self.column, self.value)),
            recovered_key: None,
            key_identity_unproven: false,
            has_predicate: true,
        }
    }
}

fn equality(expr: &Expr, qualifier: &str, positions: PlaceholderPositions<'_>) -> Option<Key> {
    let Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    } = unwrap_expr(expr)
    else {
        return None;
    };
    let pair = |column_expr: &Expr, value_expr: &Expr| {
        let column = column(column_expr, Some(qualifier), positions)?;
        let value = plain_value(value_expr)?.to_string();
        Some(Key { column, value })
    };
    pair(left, right).or_else(|| pair(right, left))
}

fn plain_table(table: &TableFactor) -> Option<bool> {
    let TableFactor::Table {
        alias,
        args: None,
        sample: None,
        inheritance,
        ..
    } = table
    else {
        return None;
    };
    if alias
        .as_ref()
        .is_some_and(|alias| !alias.columns.is_empty())
    {
        return None;
    }
    match inheritance {
        None => Some(false),
        Some(TableInheritance::Only { .. }) => Some(true),
        Some(TableInheritance::IncludeDescendants) => None,
    }
}
