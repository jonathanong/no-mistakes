//! PostgreSQL accepts ANALYZE VERBOSE and comma-separated relation/column
//! lists, while sqlparser 0.63 represents only one relation per statement.
use super::super::lenient::pg_ident;
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn rewrite(code: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    let first = code.first()?;
    if !matches!(&first.token, Token::Word(word)
        if word.quote_style.is_none() && word.keyword == Keyword::ANALYZE)
    {
        return None;
    }
    let verbose = code.get(1).is_some_and(|token| {
        matches!(&token.token, Token::Word(word)
            if word.quote_style.is_none() && word.keyword == Keyword::VERBOSE)
    });
    let mut at = 1 + usize::from(verbose);
    // VERBOSE only changes server output and has no field in the Analyze AST.
    // A quoted "VERBOSE" remains a relation name, including its column list.
    let mut result = vec![first.clone()];
    if verbose && at == code.len() {
        return Some(result);
    }
    let mut count = 0;
    loop {
        let start = at;
        relation(code, &mut at)?;
        if code
            .get(at)
            .is_some_and(|token| token.token == Token::LParen)
        {
            columns(code, &mut at)?;
        }
        result.extend_from_slice(&code[start..at]);
        count += 1;
        let Some(comma) = code.get(at) else { break };
        if comma.token != Token::Comma {
            return None;
        }
        result.push(TokenWithSpan::new(Token::SemiColon, comma.span));
        result.push(TokenWithSpan::new(
            Token::make_keyword("ANALYZE"),
            comma.span,
        ));
        at += 1;
    }
    if count < 2 && !verbose {
        return None;
    }
    // Validate the complete replacement: unknown options, incomplete targets,
    // empty column lists, and trailing SQL must never disappear in a rewrite.
    let parsed = Parser::new(&PostgreSqlDialect {})
        .with_tokens_with_locations(result.clone())
        .parse_statements()
        .ok()?;
    (parsed.len() == count
        && parsed
            .iter()
            .all(|item| matches!(item, Statement::Analyze(_))))
    .then_some(result)
}

fn relation(code: &[TokenWithSpan], at: &mut usize) -> Option<()> {
    if !pg_ident::col_id(&code.get(*at)?.token) {
        return None;
    }
    *at += 1;
    if code
        .get(*at)
        .is_some_and(|token| token.token == Token::Period)
    {
        *at += 1;
        if !pg_ident::col_id(&code.get(*at)?.token) {
            return None;
        }
        *at += 1;
    }
    Some(())
}

fn columns(code: &[TokenWithSpan], at: &mut usize) -> Option<()> {
    *at += 1; // Opening parenthesis, checked by the caller.
    loop {
        if !pg_ident::col_id(&code.get(*at)?.token) {
            return None;
        }
        *at += 1;
        match code.get(*at)?.token {
            Token::Comma => *at += 1,
            Token::RParen => {
                *at += 1;
                return Some(());
            }
            _ => return None,
        }
    }
}
