use super::super::evaluate::Value;
use super::ProjectedSql;
use crate::codebase::postgres::embedded::publish_placeholders_with_positions;
use crate::codebase::postgres::sql_requires_query_annotation;

pub(super) fn projected_sql(value: &Value) -> Option<Vec<ProjectedSql>> {
    match value {
        Value::Prefix(text, complete, _) => {
            let (text, positions) = publish_placeholders_with_positions(text.clone());
            let generated_bind = !positions.is_empty();
            Some(vec![ProjectedSql {
                text,
                complete: *complete && !generated_bind,
                only_value_placeholders: *complete && generated_bind,
            }])
        }
        Value::Promise(value) | Value::Evaluated(value, _) => projected_sql(value),
        Value::Joined(values) if !values.is_empty() => {
            let mut alternatives = Vec::new();
            for value in values.iter() {
                for candidate in projected_sql(value)? {
                    if !alternatives.contains(&candidate) {
                        alternatives.push(candidate);
                    }
                }
            }
            Some(alternatives)
        }
        _ => None,
    }
}

pub(super) fn prefix(value: Value) -> Option<String> {
    if let Value::Possible(values) = value {
        // A known unsafe possibility still violates the rule. A known safe
        // possibility cannot establish safety for the implicit unknown case.
        return values
            .into_iter()
            .filter_map(prefix)
            .find(|text| sql_requires_query_annotation(text));
    }
    let Value::Prefix(text, complete, _) = value else {
        return None;
    };
    let start = text.trim_start();
    if complete
        || super::super::super::annotation::has_leading_query_annotation(&text)
        || (!start.is_empty()
            && !start.starts_with('/')
            && !could_be_transaction(start)
            && sql_requires_query_annotation(&text))
        || (start.starts_with('/') && start.len() > 1 && !start.starts_with("/*"))
        || (start.starts_with("/*") && start.contains("*/") && sql_requires_query_annotation(&text))
    {
        Some(text)
    } else {
        None
    }
}

fn could_be_transaction(prefix: &str) -> bool {
    let prefix = prefix.to_ascii_uppercase();
    ["BEGIN", "COMMIT", "ROLLBACK"]
        .iter()
        .any(|statement| statement.starts_with(&prefix))
}
