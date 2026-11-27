use super::word;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn rewrite(code: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(code.to_vec());
    if !parser.parse_keyword(Keyword::CREATE) {
        return None;
    }
    let _ = parser.parse_keywords(&[Keyword::OR, Keyword::REPLACE]);
    let _ = parser.parse_keyword(Keyword::CONSTRAINT);
    if !parser.parse_keyword(Keyword::TRIGGER) {
        return None;
    }
    let at = code.windows(2).position(|pair| {
        word(&pair[0], "EXECUTE") && (word(&pair[1], "FUNCTION") || word(&pair[1], "PROCEDURE"))
    })? + 2;
    let mut args =
        Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(code[at..].to_vec());
    args.parse_object_name(false).ok()?;
    args.expect_token(&Token::LParen).ok()?;
    if !args.consume_token(&Token::RParen) {
        args.parse_comma_separated(Parser::parse_literal_string)
            .ok()?;
        args.expect_token(&Token::RParen).ok()?;
    }
    if args.peek_token().token != Token::EOF {
        return None;
    }
    let open = code[at..]
        .iter()
        .position(|token| token.token == Token::LParen)?
        + at;
    // sqlparser models trigger arguments as function parameter *types*, whereas
    // PostgreSQL accepts string constants. Validate then omit only those values;
    // retain the native trigger, function, relation, events, and WHEN expression.
    let rewritten: Vec<_> = code[..=open].iter().chain(code.last()).cloned().collect();
    let mut check =
        Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(rewritten.clone());
    check.parse_statement().ok()?;
    (check.peek_token().token == Token::EOF).then_some(rewritten)
}
