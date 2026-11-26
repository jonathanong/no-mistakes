use sqlparser::tokenizer::{Token, TokenWithSpan};

// PostgreSQL accepts BETWEEN SYMMETRIC; sqlparser can validate the remaining query.
pub(super) fn erase_on_parser_clone(tokens: &mut Vec<TokenWithSpan>) {
    while let Some(at) = tokens.windows(2).position(|pair| {
        matches!(&pair[0].token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("BETWEEN"))
            && matches!(&pair[1].token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("SYMMETRIC"))
    }) {
        tokens.remove(at + 1);
    }
}
