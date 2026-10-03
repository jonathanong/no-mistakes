//! Stable predicate comparison preserves literal and quoted identifier contents.
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};

pub(crate) fn normalize(text: &str) -> String {
    let Ok(tokens) = Tokenizer::new(&PostgreSqlDialect {}, text).tokenize() else {
        return text.to_string();
    };
    tokens
        .into_iter()
        .map(|token| match token {
            Token::Whitespace(_) => " ".to_string(),
            Token::Word(mut word) if word.quote_style.is_none() => {
                word.value = word.value.to_ascii_lowercase();
                Token::Word(word).to_string()
            }
            other => other.to_string(),
        })
        .collect::<String>()
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests;
