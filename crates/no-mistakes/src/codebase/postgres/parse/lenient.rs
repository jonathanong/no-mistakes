use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, Tokenizer, Word};

mod recover;

/// Tokenize, rewrite PG18 virtual generated columns, then parse each statement.
///
/// Unparseable `DO $tag$ … $tag$` statements are peeled so schema DDL inside
/// the body can still parse. Remaining unparseable chunks recover `ALTER TABLE`,
/// `CREATE TABLE`, and `CREATE [UNIQUE] INDEX` after PL/pgSQL wrappers.
pub(super) fn parse_postgres_sql_lenient(sql: &str) -> Vec<Statement> {
    let dialect = PostgreSqlDialect {};
    let Ok(mut tokens) = Tokenizer::new(&dialect, sql).tokenize() else {
        return Vec::new();
    };
    rewrite_virtual_generated_columns(&mut tokens);
    recover::parse_chunks(split_statement_tokens(tokens))
}

fn rewrite_virtual_generated_columns(tokens: &mut Vec<Token>) {
    let mut index = 0;
    while index < tokens.len() {
        if keyword_of(&tokens[index]) == Some(Keyword::GENERATED) {
            if let Some(after_expr) = skip_generated_always_as_expr(tokens, index) {
                apply_stored_generated_mode(tokens, after_expr);
            }
        }
        index += 1;
    }
}

fn apply_stored_generated_mode(tokens: &mut Vec<Token>, after_expr: usize) {
    match next_non_ws(tokens, after_expr) {
        Some(idx) => match keyword_of(&tokens[idx]) {
            Some(Keyword::VIRTUAL) => set_stored(&mut tokens[idx]),
            Some(Keyword::STORED) => {}
            _ => tokens.insert(idx, stored_token()),
        },
        None => tokens.push(stored_token()),
    }
}

fn skip_generated_always_as_expr(tokens: &[Token], generated_at: usize) -> Option<usize> {
    let always_at = skip_ws(tokens, generated_at + 1);
    if keyword_of(tokens.get(always_at)?) != Some(Keyword::ALWAYS) {
        return None;
    }
    let as_at = skip_ws(tokens, always_at + 1);
    if keyword_of(tokens.get(as_at)?) != Some(Keyword::AS) {
        return None;
    }
    let open_at = skip_ws(tokens, as_at + 1);
    if !matches!(tokens.get(open_at)?, Token::LParen) {
        return None;
    }
    skip_balanced_parens(tokens, open_at)
}

fn skip_balanced_parens(tokens: &[Token], open_at: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open_at;
    while index < tokens.len() {
        match &tokens[index] {
            Token::LParen => depth += 1,
            Token::RParen => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn split_statement_tokens(tokens: Vec<Token>) -> Vec<Vec<Token>> {
    let mut statements = Vec::new();
    let mut current = Vec::new();
    for token in tokens {
        if matches!(token, Token::SemiColon) {
            if has_non_ws(&current) {
                statements.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        } else {
            current.push(token);
        }
    }
    if has_non_ws(&current) {
        statements.push(current);
    }
    statements
}

pub(super) fn skip_ws(tokens: &[Token], mut index: usize) -> usize {
    while index < tokens.len() && matches!(tokens[index], Token::Whitespace(_)) {
        index += 1;
    }
    index
}

pub(super) fn next_non_ws(tokens: &[Token], start: usize) -> Option<usize> {
    let index = skip_ws(tokens, start);
    (index < tokens.len()).then_some(index)
}

fn has_non_ws(tokens: &[Token]) -> bool {
    tokens
        .iter()
        .any(|token| !matches!(token, Token::Whitespace(_)))
}

pub(super) fn keyword_of(token: &Token) -> Option<Keyword> {
    match token {
        Token::Word(word) => Some(word.keyword),
        _ => None,
    }
}

fn set_stored(token: &mut Token) {
    if let Token::Word(word) = token {
        word.value = "STORED".to_string();
        word.keyword = Keyword::STORED;
    }
}

fn stored_token() -> Token {
    Token::Word(Word {
        value: "STORED".to_string(),
        quote_style: None,
        keyword: Keyword::STORED,
    })
}

#[cfg(test)]
mod tests;
