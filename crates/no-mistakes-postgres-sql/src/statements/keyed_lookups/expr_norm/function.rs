use super::super::builtins::SAFE_PURE_FUNCTIONS;
use crate::idents::ident_key;
use sqlparser::ast::{
    Expr, Function, FunctionArg, FunctionArgExpr, FunctionArguments, ObjectNamePart,
};

pub(super) fn normalize(
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
