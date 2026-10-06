//! Recover SQL expression statements through narrow PL/pgSQL prefixes.
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};

pub(super) fn recover(tokens: &[Token], original: Option<&[TokenWithSpan]>) -> Vec<Statement> {
    let mut at = next(tokens, 0);
    let begin = tokens
        .get(at)
        .is_some_and(|token| word(token, "BEGIN") || word(token, "ELSE"));
    if begin {
        at = next(tokens, at + 1);
    }
    let mut result = Vec::new();
    let guarded = tokens
        .get(at)
        .is_some_and(|token| word(token, "IF") || word(token, "ELSIF"));
    if guarded {
        let Some(then) = condition_end(tokens, at + 1) else {
            return result;
        };
        if let Some(condition) = select(
            &tokens[at + 1..then],
            original.map(|tokens| &tokens[at + 1..then]),
            true,
        ) {
            result.push(condition);
        }
        at = next(tokens, then + 1);
    }
    let Some(first) = tokens.get(at) else {
        return result;
    };
    let mut replace = word(first, "PERFORM") || word(first, "RETURN");
    let query = word(first, "SELECT") || word(first, "WITH");
    if !(replace || query && (begin || guarded)) {
        return result;
    }
    let mut start = if replace { next(tokens, at + 1) } else { at };
    if word(first, "RETURN") && tokens.get(start).is_some_and(|token| word(token, "QUERY")) {
        start = next(tokens, start + 1);
        replace = false;
    }
    if let Some(statement) = select(
        &tokens[start..],
        original.map(|tokens| &tokens[start..]),
        replace,
    ) {
        result.push(statement);
    }
    result
}

fn select(
    tokens: &[Token],
    original: Option<&[TokenWithSpan]>,
    prepend: bool,
) -> Option<Statement> {
    let dialect = PostgreSqlDialect {};
    let mut located = original.map_or_else(
        || tokens.iter().cloned().map(TokenWithSpan::wrap).collect(),
        |tokens| tokens.to_vec(),
    );
    if prepend {
        let token = Token::Word(Word {
            value: "SELECT".into(),
            quote_style: None,
            keyword: Keyword::SELECT,
        });
        let span = located.first()?.span;
        located.insert(0, TokenWithSpan::new(token, span));
    }
    Parser::new(&dialect)
        .with_tokens_with_locations(located)
        .parse_statement()
        .ok()
}

fn next(tokens: &[Token], at: usize) -> usize {
    tokens
        .iter()
        .enumerate()
        .skip(at)
        .find(|(_, token)| !matches!(token, Token::Whitespace(_)))
        .map_or(tokens.len(), |(at, _)| at)
}

fn condition_end(tokens: &[Token], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut cases = 0usize;
    for (at, token) in tokens.iter().enumerate().skip(start) {
        match token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
        if word(token, "CASE") {
            cases += 1;
        }
        if word(token, "END") {
            cases = cases.saturating_sub(1);
        }
        if depth == 0 && cases == 0 && word(token, "THEN") {
            return Some(at);
        }
    }
    None
}

fn word(token: &Token, name: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests;
