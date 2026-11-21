//! A `\.` line ends COPY stdin data. It is not the next statement.
use sqlparser::{parser::Parser, tokenizer::Token};

pub(super) fn skip(parser: &mut Parser<'_>) {
    loop {
        if parser.consume_token(&Token::SemiColon) {
            continue;
        }
        if !terminator(parser) {
            break;
        }
    }
}

fn terminator(parser: &mut Parser<'_>) -> bool {
    if parser.peek_token().token != Token::Backslash
        || parser.peek_nth_token(1).token != Token::Period
    {
        return false;
    }
    parser.next_token();
    parser.next_token();
    true
}
