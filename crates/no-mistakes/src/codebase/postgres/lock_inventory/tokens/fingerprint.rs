use super::Lexeme;
use sqlparser::tokenizer::Token;
use std::fmt::Write;

pub(super) fn normalize(tokens: &[Lexeme<'_>]) -> String {
    let mut normalized = String::new();
    for token in tokens {
        let lexeme = match token.token() {
            Token::Word(word) if word.quote_style.is_none() => word.value.to_ascii_lowercase(),
            other => other.to_string(),
        };
        write!(normalized, "{}:{lexeme}", lexeme.len()).expect("writing to a string succeeds");
    }
    normalized
}
