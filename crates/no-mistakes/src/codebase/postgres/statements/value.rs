use super::{SqlAssignmentFact, SqlValueForm};
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::schema::relation_name;
use sqlparser::ast::{
    Assignment, AssignmentTarget, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments,
    ObjectNamePart, Value, ValueWithSpan,
};

const VOLATILE: &[&str] = &[
    "gen_random_uuid",
    "random",
    "nextval",
    "uuidv7",
    "uuid_generate_v1",
    "uuid_generate_v1mc",
    "uuid_generate_v4",
    "now",
    "clock_timestamp",
    "statement_timestamp",
    "transaction_timestamp",
    "current_timestamp",
    "current_date",
    "current_time",
    "localtime",
    "localtimestamp",
];

pub(super) fn from_assignment(assignment: &Assignment) -> SqlAssignmentFact {
    SqlAssignmentFact {
        column: assignment_column(&assignment.target),
        form: from_expr(&assignment.value),
    }
}

pub(super) fn from_expr(expr: &Expr) -> SqlValueForm {
    match unwrap_expr(expr) {
        Expr::Value(value) => from_value(value),
        Expr::Identifier(ident) if is_placeholder_ident(&ident.value) => SqlValueForm::Placeholder,
        Expr::Identifier(ident) if is_volatile_name(&ident.value) => SqlValueForm::Volatile {
            name: ident.value.to_ascii_lowercase(),
        },
        Expr::Identifier(ident) if ident.value.eq_ignore_ascii_case("default") => {
            SqlValueForm::Other
        }
        Expr::Identifier(ident) => SqlValueForm::SelfRef {
            column: ident.value.clone(),
        },
        Expr::CompoundIdentifier(parts) => compound_form(parts),
        Expr::Function(function) => from_function(function),
        Expr::Subquery(_) | Expr::Exists { .. } | Expr::InSubquery { .. } => SqlValueForm::Subquery,
        _ => SqlValueForm::Other,
    }
}

fn from_value(value: &ValueWithSpan) -> SqlValueForm {
    match &value.value {
        Value::Null => SqlValueForm::Null,
        Value::Placeholder(_) => SqlValueForm::Placeholder,
        _ => SqlValueForm::Literal,
    }
}

fn compound_form(parts: &[sqlparser::ast::Ident]) -> SqlValueForm {
    if parts.len() >= 2 && parts[0].value.eq_ignore_ascii_case("excluded") {
        return SqlValueForm::Excluded {
            column: parts[parts.len() - 1].value.clone(),
        };
    }
    SqlValueForm::SelfRef {
        column: parts[parts.len() - 1].value.clone(),
    }
}

fn from_function(function: &Function) -> SqlValueForm {
    let name = last_function_name(function).to_ascii_lowercase();
    if is_volatile_name(&name) {
        return SqlValueForm::Volatile { name };
    }
    let args = function_arg_exprs(function)
        .into_iter()
        .map(from_expr)
        .collect();
    match name.as_str() {
        "coalesce" => SqlValueForm::Coalesce { args },
        "greatest" => SqlValueForm::Greatest { args },
        "least" => SqlValueForm::Least { args },
        _ => SqlValueForm::Other,
    }
}

fn last_function_name(function: &Function) -> String {
    function
        .name
        .0
        .iter()
        .rev()
        .find_map(|part| match part {
            ObjectNamePart::Identifier(ident) => Some(ident.value.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

fn function_arg_exprs(function: &Function) -> Vec<&Expr> {
    let FunctionArguments::List(list) = &function.args else {
        return Vec::new();
    };
    list.args.iter().filter_map(named_or_unnamed_expr).collect()
}

fn named_or_unnamed_expr(arg: &FunctionArg) -> Option<&Expr> {
    match arg {
        FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))
        | FunctionArg::Named {
            arg: FunctionArgExpr::Expr(expr),
            ..
        }
        | FunctionArg::ExprNamed {
            arg: FunctionArgExpr::Expr(expr),
            ..
        } => Some(expr),
        _ => None,
    }
}

fn assignment_column(target: &AssignmentTarget) -> String {
    match target {
        AssignmentTarget::ColumnName(name) => relation_name(name),
        AssignmentTarget::Tuple(names) => names
            .first()
            .map(relation_name)
            .unwrap_or_else(|| "?".to_string()),
    }
}

pub(super) fn self_ref_column(expr: &Expr) -> Option<String> {
    match from_expr(expr) {
        SqlValueForm::SelfRef { column } => Some(column),
        _ => None,
    }
}

pub(super) fn excluded_column(expr: &Expr) -> Option<String> {
    match from_expr(expr) {
        SqlValueForm::Excluded { column } => Some(column),
        _ => None,
    }
}

pub(super) fn is_placeholder_ident(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with('$') || lower.starts_with("sql_placeholder_")
}

fn is_volatile_name(name: &str) -> bool {
    VOLATILE.iter().any(|item| name.eq_ignore_ascii_case(item))
}

pub(crate) fn form_is_stable(form: &SqlValueForm) -> bool {
    match form {
        SqlValueForm::Literal | SqlValueForm::Null | SqlValueForm::Placeholder => true,
        SqlValueForm::Excluded { .. }
        | SqlValueForm::SelfRef { .. }
        | SqlValueForm::Volatile { .. }
        | SqlValueForm::Subquery
        | SqlValueForm::Other => false,
        SqlValueForm::Coalesce { args }
        | SqlValueForm::Greatest { args }
        | SqlValueForm::Least { args } => args.iter().all(form_is_stable),
    }
}
