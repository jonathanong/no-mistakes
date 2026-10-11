use super::{RuleFinding, RULE_ID};

pub(in crate::codebase::rules::postgres_sql_shape_policy) fn finding(
    file: &str,
    line: usize,
    message: &str,
    target: &str,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}
