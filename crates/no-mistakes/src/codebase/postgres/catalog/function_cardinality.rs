use super::names::decoded_parts;
use sqlparser::ast::{FunctionReturnType, Statement};
use sqlparser::{dialect::PostgreSqlDialect, parser::Parser};

pub(super) fn resolve(
    metadata: Option<bool>,
    definition: &str,
    name: &str,
    contract: &str,
    schema: Option<&str>,
) -> Option<bool> {
    if matches!(contract.split_whitespace().next(), Some("setof" | "table")) {
        return Some(true);
    }
    if metadata.is_some() {
        return metadata;
    }
    let statements = Parser::parse_sql(&PostgreSqlDialect {}, definition).ok()?;
    let [Statement::CreateFunction(function)] = statements.as_slice() else {
        return None;
    };
    let expected = qualify(decoded_parts(name), schema)?;
    let actual = qualify(decoded_parts(&function.name.to_string()), schema)?;
    if actual != expected {
        return None;
    }
    match &function.return_type {
        Some(FunctionReturnType::DataType(_)) => Some(false),
        Some(FunctionReturnType::SetOf(_)) => Some(true),
        None => None,
    }
}

fn qualify(parts: Vec<String>, schema: Option<&str>) -> Option<Vec<String>> {
    match (parts.as_slice(), schema) {
        ([name], Some(schema)) => Some(vec![schema.to_string(), name.clone()]),
        ([_] | [_, _], _) => Some(parts),
        _ => None,
    }
}
