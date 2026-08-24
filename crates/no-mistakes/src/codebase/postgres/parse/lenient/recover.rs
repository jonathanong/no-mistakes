use super::{keyword_of, next_non_ws, skip_ws};
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Token;

pub(super) fn parse_chunks(chunks: Vec<Vec<Token>>) -> Vec<Statement> {
    chunks.into_iter().flat_map(parse_chunk).collect()
}

fn parse_chunk(chunk: Vec<Token>) -> Vec<Statement> {
    if let Some(body) = peel_do_body(&chunk) {
        return super::parse_postgres_sql_lenient(&body)
            .into_iter()
            .filter(|statement| !is_begin_or_end(statement))
            .collect();
    }
    let dialect = PostgreSqlDialect {};
    let mut parser = Parser::new(&dialect).with_tokens(chunk.clone());
    match parser.parse_statement() {
        Ok(statement) if matches!(parser.peek_token().token, Token::EOF) => vec![statement],
        _ => {
            let recovered = recover_chr_encoded(&chunk);
            if recovered.is_empty() {
                recover_schema_ddl(&chunk).into_iter().collect()
            } else {
                recovered
            }
        }
    }
}

fn is_begin_or_end(statement: &Statement) -> bool {
    matches!(
        statement,
        Statement::StartTransaction { .. } | Statement::Commit { .. }
    )
}

fn peel_do_body(tokens: &[Token]) -> Option<String> {
    let mut index = skip_ws(tokens, 0);
    if keyword_of(tokens.get(index)?) != Some(Keyword::DO) {
        return None;
    }
    index = skip_ws(tokens, index + 1);
    if keyword_of(tokens.get(index)?) == Some(Keyword::LANGUAGE) {
        index = skip_ws(tokens, index + 1);
        if !matches!(tokens.get(index)?, Token::Word(_)) {
            return None;
        }
        index = skip_ws(tokens, index + 1);
    }
    match tokens.get(index)? {
        Token::DollarQuotedString(body) => {
            let rest = skip_ws(tokens, index + 1);
            (rest >= tokens.len()).then(|| body.value.clone())
        }
        _ => None,
    }
}

fn recover_chr_encoded(tokens: &[Token]) -> Vec<Statement> {
    let mut rewritten = tokens.to_vec();
    super::rewrite_chr_tokens(&mut rewritten);
    concatenated_strings(&rewritten)
        .map(|sql| super::parse_postgres_sql_lenient(&sql))
        .unwrap_or_default()
}

pub(super) fn concatenated_strings(tokens: &[Token]) -> Option<String> {
    let mut sql = String::new();
    let mut expect_string = true;
    let mut saw_string = false;
    for token in tokens {
        if matches!(token, Token::Whitespace(_)) {
            continue;
        }
        match token {
            Token::SingleQuotedString(value) if expect_string => {
                sql.push_str(value);
                expect_string = false;
                saw_string = true;
            }
            Token::DollarQuotedString(value) if expect_string => {
                sql.push_str(&value.value);
                expect_string = false;
                saw_string = true;
            }
            Token::StringConcat if !expect_string => expect_string = true,
            _ => return None,
        }
    }
    (saw_string && !expect_string && !sql.is_empty()).then_some(sql)
}

fn recover_schema_ddl(tokens: &[Token]) -> Option<Statement> {
    let start = schema_ddl_start(tokens)?;
    Parser::new(&PostgreSqlDialect {})
        .with_tokens(tokens[start..].to_vec())
        .parse_statement()
        .ok()
}

fn schema_ddl_start(tokens: &[Token]) -> Option<usize> {
    let mut index = 0;
    while index < tokens.len() {
        if let Some(start) = ddl_start_at(tokens, index) {
            return Some(start);
        }
        index += 1;
    }
    None
}

fn ddl_start_at(tokens: &[Token], index: usize) -> Option<usize> {
    match keyword_of(&tokens[index]) {
        Some(Keyword::ALTER) => follows_keyword(tokens, index, Keyword::TABLE).then_some(index),
        Some(Keyword::CREATE) => create_ddl_start(tokens, index),
        Some(Keyword::DROP) => drop_ddl_start(tokens, index),
        Some(Keyword::TRUNCATE) => Some(index),
        _ => None,
    }
}

fn create_ddl_start(tokens: &[Token], index: usize) -> Option<usize> {
    let next = next_non_ws(tokens, index + 1)?;
    match keyword_of(&tokens[next]) {
        Some(Keyword::TABLE) | Some(Keyword::INDEX) | Some(Keyword::VIEW) => Some(index),
        Some(Keyword::UNIQUE) => follows_keyword(tokens, next, Keyword::INDEX).then_some(index),
        Some(Keyword::MATERIALIZED) => {
            follows_keyword(tokens, next, Keyword::VIEW).then_some(index)
        }
        _ => None,
    }
}

fn drop_ddl_start(tokens: &[Token], index: usize) -> Option<usize> {
    let next = next_non_ws(tokens, index + 1)?;
    match keyword_of(&tokens[next]) {
        Some(Keyword::INDEX) | Some(Keyword::TABLE) | Some(Keyword::VIEW) => Some(index),
        Some(Keyword::MATERIALIZED) => {
            follows_keyword(tokens, next, Keyword::VIEW).then_some(index)
        }
        _ => None,
    }
}

fn follows_keyword(tokens: &[Token], after: usize, expected: Keyword) -> bool {
    next_non_ws(tokens, after + 1)
        .and_then(|index| keyword_of(&tokens[index]))
        .is_some_and(|keyword| keyword == expected)
}

#[cfg(test)]
mod tests;
