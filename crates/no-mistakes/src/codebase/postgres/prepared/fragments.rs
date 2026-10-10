use crate::codebase::postgres::{EmbeddedSqlFileFacts, EmbeddedSqlKind, SqlStatementFileFacts};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub(crate) struct PreparedSqlFragment {
    pub line: u32,
    pub statements: Arc<SqlStatementFileFacts>,
}

/// Parse each unexecuted recovered builder text once within its projection.
pub(super) fn collect(file: &EmbeddedSqlFileFacts) -> Vec<PreparedSqlFragment> {
    let executed: HashSet<_> = file
        .calls
        .iter()
        .filter(|call| call.kind != EmbeddedSqlKind::Dynamic)
        .filter_map(|call| {
            call.sql_text
                .as_deref()
                .map(|sql| (sql, call.recovered_placeholder_positions.as_slice()))
        })
        .collect();
    let mut parsed = HashMap::new();
    file.fragments
        .iter()
        .filter_map(|fragment| {
            let sql = fragment.sql_text.as_deref()?;
            if executed.contains(&(sql, fragment.recovered_placeholder_positions.as_slice())) {
                return None;
            }
            let key = (
                sql.to_string(),
                fragment.recovered_placeholder_positions.clone(),
            );
            let statements = parsed.entry(key).or_insert_with(|| {
                Arc::new(statement_facts(
                    sql,
                    &fragment.recovered_placeholder_positions,
                ))
            });
            Some(PreparedSqlFragment {
                line: fragment.line,
                statements: Arc::clone(statements),
            })
        })
        .collect()
}

fn statement_facts(
    sql: &str,
    recovered_placeholder_positions: &[(u32, u32)],
) -> SqlStatementFileFacts {
    use crate::codebase::postgres::statements::extract_sql_statement_facts_with_recovered_placeholders;
    let direct = extract_sql_statement_facts_with_recovered_placeholders(
        sql,
        false,
        recovered_placeholder_positions,
    );
    if !direct.parse_failed || !direct.selects.is_empty() {
        return direct;
    }
    // Predicate-only fragments retain the legacy synthetic SELECT context.
    let prefix = sql.trim_start();
    let (prefix, wrapper) = if prefix.starts_with("AND ") || prefix.starts_with("OR ") {
        let prefix = "SELECT 1 WHERE true ";
        (prefix, format!("{prefix}{sql}"))
    } else if starts_with_clause(prefix) {
        // A fragment that is only the tail of a query (` ORDER BY id LIMIT 500`).
        let prefix = "SELECT 1 ";
        (prefix, format!("{prefix}{sql}"))
    } else {
        let prefix = "SELECT 1 WHERE ";
        (prefix, format!("{prefix}{sql}"))
    };
    let wrapped_positions = prefix_positions(recovered_placeholder_positions, prefix);
    extract_sql_statement_facts_with_recovered_placeholders(&wrapper, false, &wrapped_positions)
}

fn prefix_positions(positions: &[(u32, u32)], prefix: &str) -> Vec<(u32, u32)> {
    let prefix_lines = prefix.matches('\n').count() as u32;
    let prefix_width = prefix
        .rsplit('\n')
        .next()
        .unwrap_or_default()
        .chars()
        .count() as u32;
    positions
        .iter()
        .map(|(line, column)| {
            (
                line + prefix_lines,
                column + if *line == 1 { prefix_width } else { 0 },
            )
        })
        .collect()
}

/// Whether `text` begins with a clause that closes a query, as a word: `LIMIT 5`, not `limit_at`.
fn starts_with_clause(text: &str) -> bool {
    use sqlparser::dialect::PostgreSqlDialect;
    use sqlparser::keywords::Keyword;
    use sqlparser::tokenizer::Token;
    let Ok(tokens) =
        crate::codebase::postgres::parse::operator_boundary::tokenize(&PostgreSqlDialect {}, text)
    else {
        return false;
    };
    let mut tokens = tokens
        .iter()
        .filter(|token| !matches!(token, Token::Whitespace(_)));
    match tokens.next() {
        Some(Token::Word(word)) if word.keyword == Keyword::ORDER => {
            matches!(tokens.next(), Some(Token::Word(word)) if word.keyword == Keyword::BY)
        }
        Some(Token::Word(word)) => {
            matches!(
                word.keyword,
                Keyword::LIMIT | Keyword::OFFSET | Keyword::FETCH
            )
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests;
