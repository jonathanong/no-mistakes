use sqlparser::{parser::Parser, tokenizer::Token};

pub(super) fn recover(parser: &mut Parser<'_>, markers: &[sqlparser::tokenizer::Location]) {
    loop {
        let token = parser.peek_token();
        let boundary =
            token.token == Token::SemiColon && markers.binary_search(&token.span.start).is_err();
        if boundary || token.token == Token::EOF {
            break;
        }
        // Synthetic conflict delimiters belong to this failed statement, not
        // to the next ordinal. Only an original semicolon ends recovery.
        parser.next_token();
    }
}
