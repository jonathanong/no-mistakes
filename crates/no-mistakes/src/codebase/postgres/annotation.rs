use regex::Regex;
use std::sync::OnceLock;

/// True when executed SQL is missing a leading `/* name */` annotation.
///
/// `BEGIN` / `COMMIT` / `ROLLBACK` are exempt, including when they already
/// carry a leading block comment. Matches the Filaments runtime-query
/// contract: `/^\s*\/\*\s*\S[\s\S]*?\*\//`.
pub fn sql_requires_query_annotation(sql: &str) -> bool {
    !is_transaction_command(sql) && !has_leading_query_annotation(sql)
}

fn has_leading_query_annotation(sql: &str) -> bool {
    annotation_re().is_match(sql)
}

fn is_transaction_command(sql: &str) -> bool {
    transaction_re().is_match(sql)
}

fn annotation_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?s)^\s*/\*\s*\S.*?\*/").expect("query annotation regex"))
}

fn transaction_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?is)^\s*(?:/\*.*?\*/\s*)?(?:BEGIN|COMMIT|ROLLBACK)\b")
            .expect("transaction command regex")
    })
}

#[cfg(test)]
mod tests;
