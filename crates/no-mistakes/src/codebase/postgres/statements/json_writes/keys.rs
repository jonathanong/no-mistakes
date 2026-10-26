use crate::codebase::postgres::idents::unwrap_expr;
use sqlparser::ast::{
    BinaryOperator, Expr, FunctionArg, FunctionArgExpr, FunctionArguments, Spanned, Value,
};
use sqlparser::tokenizer::Span;

pub(super) fn collect(expr: &Expr) -> Vec<(String, Span)> {
    let mut keys = Vec::new();
    collect_into(expr, &mut keys);
    keys
}

fn collect_into(expr: &Expr, keys: &mut Vec<(String, Span)>) {
    match unwrap_expr(expr) {
        Expr::Function(function)
            if matches!(
                function.name.to_string().to_ascii_lowercase().as_str(),
                "jsonb_build_object" | "pg_catalog.jsonb_build_object"
            ) =>
        {
            let FunctionArguments::List(args) = &function.args else {
                return;
            };
            // Values can contain nested objects; only alternating literal key arguments count.
            for arg in args.args.iter().step_by(2) {
                if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg {
                    if let Some(key) = string(expr) {
                        keys.push((key, expr.span()));
                    }
                }
            }
        }
        Expr::Cast {
            expr, data_type, ..
        } if is_jsonb(&data_type.to_string()) => {
            if let Some(text) = string(expr) {
                literal(&text, expr.span(), keys);
            } else {
                collect_into(expr, keys);
            }
        }
        Expr::TypedString(value) if is_jsonb(&value.data_type.to_string()) => {
            if let Some(text) = value.value.clone().into_string() {
                literal(&text, expr.span(), keys);
            }
        }
        Expr::Case {
            conditions,
            else_result,
            ..
        } => {
            // Conditions do not become the written document; only result arms do.
            for branch in conditions {
                collect_into(&branch.result, keys);
            }
            if let Some(result) = else_result {
                collect_into(result, keys);
            }
        }
        Expr::Function(function) if function.name.to_string().eq_ignore_ascii_case("coalesce") => {
            if let FunctionArguments::List(args) = &function.args {
                for arg in &args.args {
                    if let FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) = arg {
                        collect_into(expr, keys);
                    }
                }
            }
        }
        Expr::BinaryOp {
            left,
            op: BinaryOperator::StringConcat,
            right,
        } => {
            collect_into(left, keys);
            collect_into(right, keys);
        }
        _ => {}
    }
}

fn is_jsonb(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "jsonb" | "pg_catalog.jsonb"
    )
}

fn literal(text: &str, span: Span, keys: &mut Vec<(String, Span)>) {
    // SQL escaping prevents assuming decoded JSON byte offsets are source offsets.
    // Literal documents use their token location; nested keys are deliberately skipped.
    if let Ok(serde_json::Value::Object(object)) = serde_json::from_str(text) {
        keys.extend(object.into_iter().map(|(key, _)| (key, span)));
    }
}

fn string(expr: &Expr) -> Option<String> {
    if let Expr::Cast { expr, .. } = unwrap_expr(expr) {
        return string(expr);
    }
    let Expr::Value(value) = unwrap_expr(expr) else {
        return None;
    };
    match &value.value {
        Value::SingleQuotedString(text)
        | Value::EscapedStringLiteral(text)
        | Value::UnicodeStringLiteral(text)
        | Value::NationalStringLiteral(text) => Some(text.clone()),
        Value::DollarQuotedString(value) => Some(value.value.clone()),
        _ => None,
    }
}
