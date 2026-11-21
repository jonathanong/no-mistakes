use super::dialect::PostgresSourceDialect;
use sqlparser::keywords::Keyword;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{Token, TokenWithSpan};

static DIALECT: PostgresSourceDialect = PostgresSourceDialect;

pub(super) fn parser(mut tokens: Vec<TokenWithSpan>) -> Parser<'static> {
    conceal_body_keyword(&mut tokens);
    Parser::new(&DIALECT).with_tokens_with_locations(tokens)
}

pub(super) fn accept(parser: &mut Parser<'_>) -> bool {
    if !is_keyword(&parser.peek_token().token) {
        return false;
    }
    parser.next_token();
    true
}

/// sqlparser 0.64 parses `BEGIN ATOMIC` itself. Body introducers are demoted
/// before that parse, and this still recognizes them for the custom walker.
pub(super) fn is_keyword(token: &Token) -> bool {
    let Token::Word(word) = token else {
        return false;
    };
    word.quote_style.is_none()
        && (word.keyword == Keyword::ATOMIC
            || (word.keyword == Keyword::NoKeyword && word.value.eq_ignore_ascii_case("atomic")))
}

pub(crate) fn conceal_body_keyword(tokens: &mut [TokenWithSpan]) {
    let mut begin = false;
    for token in tokens {
        let Token::Word(word) = &mut token.token else {
            if !matches!(token.token, Token::Whitespace(_)) {
                begin = false;
            }
            continue;
        };
        if word.quote_style.is_some() {
            begin = false;
            continue;
        }
        if begin && word.keyword == Keyword::ATOMIC {
            word.keyword = Keyword::NoKeyword;
            begin = false;
            continue;
        }
        begin = word.keyword == Keyword::BEGIN;
    }
}
