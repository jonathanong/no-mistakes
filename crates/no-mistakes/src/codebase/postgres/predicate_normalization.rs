//! Stable predicate comparison preserves literal and quoted identifier contents.
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::tokenizer::{Token, Tokenizer};

pub(crate) fn normalize(text: &str) -> String {
    let Ok(tokens) = Tokenizer::new(&PostgreSqlDialect {}, text)
        .with_unescape(false)
        .tokenize()
    else {
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
            // Escaped literals are decoded even when tokenizer unescaping is disabled.
            // AST value rendering restores their delimiters and backslashes safely.
            Token::EscapedStringLiteral(value) => {
                sqlparser::ast::Value::EscapedStringLiteral(value).to_string()
            }
            // Unicode escapes are decoded by the tokenizer too. Re-render the AST value so a
            // decoded quote cannot erase the original literal boundary.
            Token::UnicodeStringLiteral(value) => {
                sqlparser::ast::Value::UnicodeStringLiteral(value).to_string()
            }
            // Dollar-quote tags delimit a value but are not part of that value. Fold only the
            // tag spelling; the body remains case-sensitive.
            Token::DollarQuotedString(mut value) => {
                if let Some(tag) = &mut value.tag {
                    tag.make_ascii_lowercase();
                }
                Token::DollarQuotedString(value).to_string()
            }
            // PostgreSQL hexadecimal digits are case-insensitive; ordinary string bodies are
            // intentionally left untouched.
            Token::HexStringLiteral(value) => {
                Token::HexStringLiteral(value.to_ascii_lowercase()).to_string()
            }
            // Numeric exponent markers are case-insensitive, but other token contents should
            // retain their original spelling.
            Token::Number(mut value, long) => {
                if let Some(index) = value.find('E') {
                    value.replace_range(index..index + 1, "e");
                }
                Token::Number(value, long).to_string()
            }
            other => other.to_string(),
        })
        .fold(String::new(), |mut normalized, token| {
            // Collapse only whitespace tokens: literal/identifier contents retain their spacing.
            if token != " " || !normalized.ends_with(' ') {
                normalized.push_str(&token);
            }
            normalized
        })
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests;
