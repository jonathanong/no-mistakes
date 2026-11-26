use super::builtins::SAFE_PURE_FUNCTIONS;
use super::scope::ScopeSnapshot;
use crate::codebase::postgres::idents::{ident_key, object_name_identity, unwrap_expr};
use sqlparser::ast::{
    ArrayElemTypeDef, DataType, Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments,
    ObjectNamePart, OrderByExpr,
};

pub(super) fn normalize_expr(
    expr: &Expr,
    self_qualifier: Option<&str>,
    single_table_subquery: bool,
    outer: &ScopeSnapshot,
) -> Option<String> {
    match unwrap_expr(expr) {
        Expr::CompoundIdentifier(parts) if parts.len() == 2 => {
            let qual = ident_key(&parts[0]);
            let col = ident_key(&parts[1]);
            if self_qualifier == Some(qual.as_str()) {
                return Some(format!("self.{col}"));
            }
            let binding = outer.resolve_qualifier(&qual)?;
            Some(format!("b{binding}.{col}"))
        }
        Expr::Identifier(ident) if single_table_subquery => {
            Some(format!("self.{}", ident_key(ident)))
        }
        Expr::Value(value) => Some(format!("val:{value}")),
        Expr::Cast {
            expr, data_type, ..
        } => {
            let inner = normalize_expr(expr, self_qualifier, single_table_subquery, outer)?;
            Some(format!("cast({inner},{})", canonical_type(data_type)))
        }
        Expr::TypedString(typed) => Some(format!(
            "typed({},{})",
            canonical_type(&typed.data_type),
            typed.value
        )),
        Expr::IsNull(inner) => Some(format!(
            "is_null({})",
            normalize_expr(inner, self_qualifier, single_table_subquery, outer)?
        )),
        Expr::IsNotNull(inner) => Some(format!(
            "is_not_null({})",
            normalize_expr(inner, self_qualifier, single_table_subquery, outer)?
        )),
        Expr::IsTrue(inner) => Some(format!(
            "is_true({})",
            normalize_expr(inner, self_qualifier, single_table_subquery, outer)?
        )),
        Expr::IsFalse(inner) => Some(format!(
            "is_false({})",
            normalize_expr(inner, self_qualifier, single_table_subquery, outer)?
        )),
        Expr::IsDistinctFrom(left, right) => Some(format!(
            "is_distinct({},{})",
            normalize_expr(left, self_qualifier, single_table_subquery, outer)?,
            normalize_expr(right, self_qualifier, single_table_subquery, outer)?
        )),
        Expr::IsNotDistinctFrom(left, right) => Some(format!(
            "is_not_distinct({},{})",
            normalize_expr(left, self_qualifier, single_table_subquery, outer)?,
            normalize_expr(right, self_qualifier, single_table_subquery, outer)?
        )),
        Expr::UnaryOp { op, expr } => Some(format!(
            "unary({op},{})",
            normalize_expr(expr, self_qualifier, single_table_subquery, outer)?
        )),
        Expr::BinaryOp { left, op, right } => Some(format!(
            "bin({op},{},{})",
            normalize_expr(left, self_qualifier, single_table_subquery, outer)?,
            normalize_expr(right, self_qualifier, single_table_subquery, outer)?
        )),
        Expr::In {
            expr,
            target: sqlparser::ast::InTarget::List(list),
            negated,
            ..
        } => {
            let target = normalize_expr(expr, self_qualifier, single_table_subquery, outer)?;
            let mut items = list
                .iter()
                .map(|item| normalize_expr(item, self_qualifier, single_table_subquery, outer))
                .collect::<Option<Vec<_>>>()?;
            items.sort();
            Some(format!("in({negated},{target},[{}])", items.join(",")))
        }
        Expr::Function(function) => normalize_function(function, |arg| {
            normalize_expr(arg, self_qualifier, single_table_subquery, outer)
        }),
        _ => None,
    }
}

pub(super) fn normalize_function(
    function: &Function,
    mut norm_arg: impl FnMut(&Expr) -> Option<String>,
) -> Option<String> {
    if function.over.is_some() || function.filter.is_some() || !function.within_group.is_empty() {
        return None;
    }
    let parts: Vec<String> = function
        .name
        .0
        .iter()
        .map(|part| match part {
            ObjectNamePart::Identifier(ident) => Some(ident_key(ident)),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let base = match parts.as_slice() {
        [name] => name.as_str(),
        [schema, name] if schema == "pg_catalog" => name.as_str(),
        _ => return None,
    };
    if !SAFE_PURE_FUNCTIONS.contains(&base) {
        return None;
    }
    let FunctionArguments::List(list) = &function.args else {
        return None;
    };
    if !list.clauses.is_empty() || list.duplicate_treatment.is_some() {
        return None;
    }
    let args = list
        .args
        .iter()
        .map(|arg| match arg {
            FunctionArg::Unnamed(FunctionArgExpr::Expr(expr)) => norm_arg(expr),
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    Some(format!("fn({base},{})", args.join(",")))
}

pub(super) fn canonical_type(data_type: &DataType) -> String {
    match data_type {
        DataType::Custom(name, mods) => {
            let base = object_name_identity(name);
            if mods.is_empty() {
                base
            } else {
                let mods = mods
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{base}({mods})")
            }
        }
        DataType::Array(
            ArrayElemTypeDef::AngleBracket(inner)
            | ArrayElemTypeDef::SquareBracket(inner, _)
            | ArrayElemTypeDef::Parenthesis(inner)
            | ArrayElemTypeDef::Qualified(inner, _),
        ) => format!("{}[]", canonical_type(inner)),
        other => other.to_string().to_ascii_lowercase(),
    }
}

pub(super) fn normalize_order_by(
    order_by: &[OrderByExpr],
    self_qualifier: &str,
    single_table_subquery: bool,
    outer: &ScopeSnapshot,
) -> Option<Vec<String>> {
    order_by
        .iter()
        .map(|item| {
            let expr = normalize_expr(
                &item.expr,
                Some(self_qualifier),
                single_table_subquery,
                outer,
            )?;
            Some(format!(
                "{expr}:{:?}:{:?}",
                item.options.sort, item.options.nulls_first
            ))
        })
        .collect()
}
