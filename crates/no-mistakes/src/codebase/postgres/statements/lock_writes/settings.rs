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
    !matches!(&function.args, FunctionArguments::List(arguments)
        if matches!(arguments.args.first(),
            Some(FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Value(value))))
            if matches!(&value.value, Value::SingleQuotedString(_)
                | Value::EscapedStringLiteral(_) | Value::DollarQuotedString(_))))
}
