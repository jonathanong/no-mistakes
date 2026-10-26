use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn owns_terminator(original: &[TokenWithSpan]) -> bool {
    super::super::super::copy_data::is_copy_stdin_tokens(original)
}

pub(super) fn is_terminator(tokens: &[TokenWithSpan]) -> bool {
    tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .map(|token| &token.token)
        .eq([&Token::Backslash, &Token::Period])
}
