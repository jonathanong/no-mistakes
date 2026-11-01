use crate::codebase::postgres::idents::{ident_key, object_name_key, unwrap_expr};
use sqlparser::ast::{
    BinaryOperator as Op, Cte, Expr, Select, SelectItem, SetExpr, SetOperator, SetQuantifier,
    TableFactor, Value,
};

pub(super) fn finite(cte: &Cte) -> bool {
    let SetExpr::SetOperation {
        op: SetOperator::Union,
        set_quantifier: SetQuantifier::All,
        left,
        right,
    } = &*cte.query.body
    else {
        return false;
    };
    let (SetExpr::Select(anchor), SetExpr::Select(step)) = (&**left, &**right) else {
        return false;
    };
    let name = ident_key(&cte.alias.name);
    let Some(alias) = recursive_alias(step, &name) else {
        return false;
    };
    if recursive_alias(anchor, &name).is_some() {
        return false;
    }
    step.projection.iter().enumerate().any(|(index, output)| {
        let expression = match output {
            SelectItem::ExprWithAlias { expr, .. } | SelectItem::UnnamedExpr(expr) => expr,
            _ => return false,
        };
        let Some(anchor_output) = anchor.projection.get(index) else {
            return false;
        };
        let (anchor_expression, implicit) = match anchor_output {
            SelectItem::ExprWithAlias { expr, alias } => (expr, Some(ident_key(alias))),
            SelectItem::UnnamedExpr(expr) => (expr, None),
            _ => return false,
        };
        let column = cte
            .alias
            .columns
            .get(index)
            .map(|column| ident_key(&column.name))
            .or(implicit);
        let Some(column) = column else { return false };
        literal_number(anchor_expression).is_some()
            && increasing(expression, &alias, &column)
            && step
                .selection
                .as_ref()
                .is_some_and(|predicate| capped(predicate, &alias, &column))
    })
}

fn recursive_alias(select: &Select, name: &str) -> Option<String> {
    let mut found = None;
    for table in select.from.iter().flat_map(|from| {
        std::iter::once(&from.relation).chain(from.joins.iter().map(|join| &join.relation))
    }) {
        if let TableFactor::Table {
            name: relation,
            alias,
            ..
        } = table
        {
            if object_name_key(relation) == name {
                if found.is_some() {
                    return None;
                }
                found = Some(
                    alias
                        .as_ref()
                        .map_or_else(|| name.to_owned(), |alias| ident_key(&alias.name)),
                );
            }
        }
    }
    found
}

fn column(expr: &Expr, alias: &str, name: &str) -> bool {
    matches!(unwrap_expr(expr), Expr::CompoundIdentifier(parts)
        if parts.len() == 2 && ident_key(&parts[0]) == alias && ident_key(&parts[1]) == name)
}

fn literal_number(expr: &Expr) -> Option<u64> {
    match unwrap_expr(expr) {
        Expr::Value(value) => match &value.value {
            Value::Number(number, _) => number.parse().ok(),
            _ => None,
        },
        _ => None,
    }
}

fn increasing(expr: &Expr, alias: &str, name: &str) -> bool {
    matches!(unwrap_expr(expr), Expr::BinaryOp { left, op: Op::Plus, right }
        if column(left, alias, name) && literal_number(right).is_some_and(|value| value > 0))
}

fn finite_value(expr: &Expr) -> bool {
    literal_number(expr).is_some()
        || matches!(unwrap_expr(expr), Expr::Value(value) if matches!(value.value, Value::Placeholder(_)))
}

fn capped(expr: &Expr, alias: &str, name: &str) -> bool {
    match unwrap_expr(expr) {
        Expr::BinaryOp {
            left,
            op: Op::And,
            right,
        } => capped(left, alias, name) || capped(right, alias, name),
        Expr::BinaryOp {
            left,
            op: Op::Lt | Op::LtEq,
            right,
        } => column(left, alias, name) && finite_value(right),
        Expr::BinaryOp {
            left,
            op: Op::Gt | Op::GtEq,
            right,
        } => finite_value(left) && column(right, alias, name),
        _ => false,
    }
}
