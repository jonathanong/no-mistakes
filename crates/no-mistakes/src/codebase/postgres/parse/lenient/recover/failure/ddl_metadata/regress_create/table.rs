use super::super::Cursor;
use sqlparser::tokenizer::Token;

pub(super) fn parse(c: &mut Cursor<'_>) -> bool {
    if !c.word("TABLE") {
        return false;
    }
    if !c.name() {
        return false;
    }
    let columns = if c.token(Token::LParen) {
        if c.token(Token::RParen) {
            true
        } else {
            names(c) && c.token(Token::RParen)
        }
    } else {
        true
    };
    if !columns {
        return false;
    }
    if c.word("WITH") {
        return c.token(Token::LParen) && c.word("OIDS") && c.token(Token::RParen);
    }
    if c.word("ON") && !c.words(&["COMMIT", "DELETE", "ROWS"]) {
        return false;
    }
    if !c.word("AS") {
        return false;
    }
    let source = if c.word("EXECUTE") {
        c.name()
            && (!c.token(Token::LParen) || (c.0.parse_expr().is_ok() && c.token(Token::RParen)))
    } else {
        c.0.parse_query()
            .is_ok_and(|query| crate::codebase::postgres::parse::lenient::pg_ast::valid(&query))
    };
    source && (!c.word("WITH") || (c.word("NO") && c.word("DATA") || c.word("DATA")))
}

fn names(c: &mut Cursor<'_>) -> bool {
    if !c.identifier() {
        return false;
    }
    while c.token(Token::Comma) {
        if !c.identifier() {
            return false;
        }
    }
    true
}
