use crate::codebase::postgres::idents::object_name_identity;
use sqlparser::ast::{Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Value};

pub(super) fn dynamic_setting_name(expr: &Expr) -> bool {
    let Expr::Function(function) = expr else {
        return false;
    };
    if !matches!(
        object_name_identity(&function.name).as_str(),
        "set_config" | "pg_catalog.set_config"
    ) {
        return false;
    }
    let FunctionArguments::List(arguments) = &function.args else {
        return true;
    };
    let Some(FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Value(value)))) =
        arguments.args.first()
    else {
        return true;
    };
    !matches!(
        &value.value,
        Value::SingleQuotedString(_)
            | Value::EscapedStringLiteral(_)
            | Value::DollarQuotedString(_)
    )
}
