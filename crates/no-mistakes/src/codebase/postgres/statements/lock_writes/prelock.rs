use super::{
    helpers::{access, column, qualifier},
    row_locks, LockWriteFacts, PlaceholderPositions, RowAccess,
};
use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{
    Assignment, BinaryOperator, Expr, GroupByExpr, LockType, Query, SelectItem, SetExpr, Statement,
    TableFactor, Update, Value,
};

mod upsert;

pub(super) fn collect(
    statement: &Statement,
    out: &mut LockWriteFacts,
    positions: PlaceholderPositions<'_>,
) {
    match statement {
        Statement::Query(query) => {
            out.prelock_attempt = query
                .locks
                .iter()
                .any(|lock| lock.lock_type == LockType::Update && lock.nonblock.is_none());
            if out.prelock_attempt {
                out.prelock = locked_key(query, positions);
            }
        }
        Statement::Update(update) => {
            out.conditional_write_keys = update_keys(update, positions).unwrap_or_default();
        }
        Statement::Insert(insert) => {
            out.conditional_write_keys
                .extend(upsert::upsert_key(insert, positions));
        }
        _ => {}
    }
}

fn locked_key(query: &Query, positions: PlaceholderPositions<'_>) -> Option<RowAccess> {
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
    if !plain_table(&table.relation) {
        return None;
    }
    let qualifier = qualifier(&table.relation)?;
    equality(select.selection.as_ref()?, &qualifier, positions)?;
    let row = row_locks::collect_legacy(query, positions, true);
    row.relation.is_some().then_some(row)
}

fn update_keys(update: &Update, positions: PlaceholderPositions<'_>) -> Option<Vec<RowAccess>> {
    if !update.table.joins.is_empty()
        || update.from.is_some()
        || !plain_assignments(&update.assignments)
    {
        return None;
    }
    let target = &update.table.relation;
    if !plain_table(target) {
        return None;
    }
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

fn plain_value(expr: &Expr) -> Option<&Value> {
    match unwrap_expr(expr) {
        Expr::Value(value)
            if matches!(
                value.value,
                Value::Number(..)
                    | Value::SingleQuotedString(_)
                    | Value::Boolean(_)
                    | Value::Placeholder(_)
            ) =>
        {
            Some(&value.value)
        }
        _ => None,
    }
}

fn plain_assignments(assignments: &[Assignment]) -> bool {
    assignments.iter().all(|assignment| {
        matches!(
            unwrap_expr(&assignment.value),
            Expr::Identifier(_) | Expr::CompoundIdentifier(_)
        ) || plain_value(&assignment.value).is_some()
    })
}

fn plain_table(table: &TableFactor) -> bool {
    matches!(
        table,
        TableFactor::Table {
            alias,
            args: None,
            sample: None,
            inheritance: None,
            ..
        } if alias.as_ref().is_none_or(|alias| alias.columns.is_empty())
    )
}
