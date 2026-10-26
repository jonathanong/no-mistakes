//! PostgreSQL's inherited-descendant marker after an ALTER TABLE name.
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn rewrite(code: &[TokenWithSpan]) -> Option<Vec<TokenWithSpan>> {
    let mut parser = Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(code.to_vec());
    if !parser.parse_keywords(&[Keyword::ALTER, Keyword::TABLE]) {
        return None;
    }
    let _ = parser.parse_keywords(&[Keyword::IF, Keyword::EXISTS]);
    let _ = parser.parse_keyword(Keyword::ONLY);
    parser.parse_object_name(false).ok()?;
    if parser.peek_token().token != Token::Mul {
        return None;
    }
    let star = code
        .iter()
        .position(|token| token.span == parser.peek_token().span)?;
    let rewritten = code
        .iter()
        .enumerate()
        .filter(|(at, _)| *at != star)
        .map(|(_, token)| token.clone())
        .collect::<Vec<_>>();
    let mut check =
        Parser::new(&PostgreSqlDialect {}).with_tokens_with_locations(rewritten.clone());
    let Statement::AlterTable(_) = check.parse_statement().ok()? else {
        return None;
    };
    (check.peek_token().token == Token::EOF).then_some(rewritten)
}
