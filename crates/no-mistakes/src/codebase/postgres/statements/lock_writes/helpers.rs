use super::*;
use crate::codebase::postgres::idents::ident_key;

type Keys = (Option<(String, String)>, Option<(String, (u32, u32))>);
type BoundValue = (Option<String>, Option<(u32, u32)>);
pub(super) fn access(
    table: &TableFactor,
    predicate: Option<&Expr>,
    positions: PlaceholderPositions<'_>,
) -> RowAccess {
    match table {
        TableFactor::Table {
            name,
            alias,
            args: None,
            ..
        } => {
            let (key, recovered_key) = predicate
                .and_then(|expr| equality(expr, qualifier(table).as_deref(), positions))
                .unwrap_or_default();
            RowAccess {
                relation: (!generated_name(name, positions)
                    && !alias
                        .as_ref()
                        .is_some_and(|alias| generated(&alias.name, positions)))
                .then(|| object_name_identity(name)),
                has_predicate: predicate.is_some(),
                key,
                recovered_key,
                key_aliases_renamed: alias
                    .as_ref()
                    .is_some_and(|alias| !alias.columns.is_empty()),
            }
        }
        _ => RowAccess {
            relation: None,
            key: None,
            recovered_key: None,
            key_aliases_renamed: false,
            has_predicate: predicate.is_some(),
        },
    }
}
pub(super) fn qualifier(table: &TableFactor) -> Option<String> {
    match table {
        TableFactor::Table { name, alias, .. } => Some(
            alias
                .as_ref()
                .map(|alias| ident_key(&alias.name))
                .unwrap_or_else(|| object_name_identity(name)),
        ),
        _ => None,
    }
}
fn equality(
    expr: &Expr,
    qualifier: Option<&str>,
    positions: PlaceholderPositions<'_>,
) -> Option<Keys> {
    let Expr::BinaryOp {
        left,
        op: BinaryOperator::Eq,
        right,
    } = unwrap_expr(expr)
    else {
        return None;
    };
    let binding = |column_expr, value_expr| {
        let column = column(column_expr, qualifier, positions)?;
        let (value, origin) = bound(value_expr, positions)?;
        let recovered_key = origin.map(|at| (column.clone(), at));
        let key = value.map(|value| (column, value));
        Some((key, recovered_key))
    };
    binding(left, right).or_else(|| binding(right, left))
}
fn column(
    expr: &Expr,
    qualifier: Option<&str>,
    positions: PlaceholderPositions<'_>,
) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::Identifier(id) if !generated(id, positions) => Some(ident_key(id)),
        Expr::CompoundIdentifier(ids)
            if ids.len() == 2
                && !ids.iter().any(|id| generated(id, positions))
                && Some(ident_key(&ids[0]).as_str()) == qualifier =>
        {
            Some(ident_key(&ids[1]))
        }
        _ => None,
    }
}
fn bound(expr: &Expr, positions: PlaceholderPositions<'_>) -> Option<BoundValue> {
    match unwrap_expr(expr) {
        Expr::Value(value) => {
            let at = (value.span.start.line as u32, value.span.start.column as u32);
            let origin = matches!(value.value, sqlparser::ast::Value::Placeholder(_))
                && positions.is_some_and(|positions| positions.contains(&at));
            Some((Some(value.to_string()), origin.then_some(at)))
        }
        Expr::Identifier(ident) if generated(ident, positions) => Some((
            None,
            Some((ident.span.start.line as u32, ident.span.start.column as u32)),
        )),
        Expr::Cast { expr, .. } => bound(expr, positions),
        _ => None,
    }
}
pub(super) fn constant(expr: &Expr) -> bool {
    match unwrap_expr(expr) {
        Expr::Value(value) => !matches!(
            value.value,
            sqlparser::ast::Value::Placeholder(_) | sqlparser::ast::Value::Null
        ),
        Expr::Cast { expr, .. } | Expr::UnaryOp { expr, .. } => constant(expr),
        Expr::BinaryOp { left, right, .. } => constant(left) && constant(right),
        _ => false,
    }
}

fn generated(ident: &sqlparser::ast::Ident, positions: PlaceholderPositions<'_>) -> bool {
    super::super::value::is_placeholder_ident_at(ident, Some(positions.unwrap_or_default()))
}

pub(super) fn generated_name(
    name: &sqlparser::ast::ObjectName,
    positions: PlaceholderPositions<'_>,
) -> bool {
    name.0
        .iter()
        .filter_map(|part| part.as_ident())
        .any(|ident| generated(ident, positions))
}
