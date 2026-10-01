use sqlparser::ast::{BinaryOperator, Expr, Value};

pub(super) fn pinned_literals(definition: &str, column: &str) -> Option<Vec<String>> {
    let expr = super::parse::expression(definition)?;
    pin(&expr, column)
}

fn pin(expr: &Expr, column: &str) -> Option<Vec<String>> {
    let expr = peel_nested(expr);
    match expr {
        Expr::BinaryOp { left, op, right } => binary(left, op, right, column),
        Expr::InList {
            expr,
            list,
            negated: false,
        } => in_list(expr, list, column),
        Expr::AnyOp {
            left,
            compare_op: BinaryOperator::Eq,
            right,
            ..
        } => any_array(left, right, column),
        Expr::IsNull(inner) if is_column(inner, column) => Some(Vec::new()),
        _ => None,
    }
}

fn binary(left: &Expr, op: &BinaryOperator, right: &Expr, column: &str) -> Option<Vec<String>> {
    match op {
        BinaryOperator::Or => combine(left, right, column, true),
        BinaryOperator::And => combine(left, right, column, false),
        BinaryOperator::Eq => equality(left, right, column),
        _ => None,
    }
}

fn combine(left: &Expr, right: &Expr, column: &str, or_branches: bool) -> Option<Vec<String>> {
    let op = if or_branches {
        BinaryOperator::Or
    } else {
        BinaryOperator::And
    };
    let mut parts = Vec::new();
    parts_of(left, &op, &mut parts);
    parts_of(right, &op, &mut parts);
    if or_branches {
        let mut union = Vec::new();
        for part in parts {
            extend_new(&mut union, &pin(part, column)?);
        }
        return Some(union);
    }
    let mut lists = Vec::new();
    for part in parts {
        if let Some(values) = pin(part, column) {
            lists.push(values);
        }
    }
    if lists.is_empty() {
        return None;
    }
    Some(first_non_empty(lists))
}

fn parts_of<'a>(expr: &'a Expr, op: &BinaryOperator, out: &mut Vec<&'a Expr>) {
    let expr = peel_nested(expr);
    if let Expr::BinaryOp {
        left,
        op: found,
        right,
    } = expr
    {
        if found == op {
            parts_of(left, op, out);
            parts_of(right, op, out);
            return;
        }
    }
    out.push(expr);
}

fn first_non_empty(lists: Vec<Vec<String>>) -> Vec<String> {
    lists
        .into_iter()
        .find(|values| !values.is_empty())
        .unwrap_or_default()
}

fn equality(left: &Expr, right: &Expr, column: &str) -> Option<Vec<String>> {
    let literal = if is_column(left, column) {
        string_literal(right)
    } else if is_column(right, column) {
        string_literal(left)
    } else {
        None
    }?;
    Some(vec![literal])
}

fn in_list(expr: &Expr, list: &[Expr], column: &str) -> Option<Vec<String>> {
    is_column(expr, column).then(|| string_list(list))?
}

fn any_array(left: &Expr, right: &Expr, column: &str) -> Option<Vec<String>> {
    if !is_column(left, column) {
        return None;
    }
    let Expr::Array(array) = peel_nested(right) else {
        return None;
    };
    string_list(&array.elem)
}

fn string_list(list: &[Expr]) -> Option<Vec<String>> {
    let mut values = Vec::new();
    for expr in list {
        extend_new(&mut values, &[string_literal(expr)?]);
    }
    Some(values)
}

fn extend_new(values: &mut Vec<String>, more: &[String]) {
    for value in more {
        if !values.iter().any(|existing| existing == value) {
            values.push(value.clone());
        }
    }
}

fn is_column(expr: &Expr, column: &str) -> bool {
    let Expr::Identifier(ident) = peel_cast(expr) else {
        return false;
    };
    if ident.quote_style == Some('"') {
        ident.value == column
    } else {
        ident.value.eq_ignore_ascii_case(column)
    }
}

fn string_literal(expr: &Expr) -> Option<String> {
    let Expr::Value(value) = peel_cast(expr) else {
        return None;
    };
    match &value.value {
        Value::SingleQuotedString(text)
        | Value::EscapedStringLiteral(text)
        | Value::NationalStringLiteral(text)
        | Value::UnicodeStringLiteral(text) => Some(text.clone()),
        Value::DollarQuotedString(text) => Some(text.value.clone()),
        _ => None,
    }
}

fn peel_nested(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) => peel_nested(inner),
        other => other,
    }
}

fn peel_cast(expr: &Expr) -> &Expr {
    match expr {
        Expr::Nested(inner) | Expr::Cast { expr: inner, .. } => peel_cast(inner),
        other => other,
    }
}
