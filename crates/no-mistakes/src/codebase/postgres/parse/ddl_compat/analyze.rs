//! PostgreSQL accepts a comma-separated ANALYZE relation list, while
//! sqlparser 0.63 represents only one relation per Analyze statement.
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
    let mut at = 1;
    let mut result = vec![first.clone()];
    let mut count = 0;
    loop {
        // This path accepts only simple, optionally qualified table names.
        // Options, column lists, and any trailing SQL remain with the parser.
        let name = code.get(at)?;
        if !pg_ident::col_id(&name.token) {
            return None;
        }
        result.push(name.clone());
        at += 1;
        if code
            .get(at)
            .is_some_and(|token| token.token == Token::Period)
        {
            result.push(code[at].clone());
            at += 1;
            let name = code.get(at)?;
            if !pg_ident::col_id(&name.token) {
                return None;
            }
            result.push(name.clone());
            at += 1;
        }
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
    if count < 2 {
        return None;
    }
    // The replacement has one Analyze AST per actual target. Verify the
    // complete token stream before publishing the parser compatibility rewrite.
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
