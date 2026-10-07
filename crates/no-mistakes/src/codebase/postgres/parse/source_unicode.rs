//! Defer Unicode literal decoding until its optional UESCAPE is in the same inventory.
use sqlparser::{
    dialect::{Dialect, PostgreSqlDialect},
    keywords::Keyword,
    tokenizer::{Token, TokenWithSpan},
};

#[derive(Debug)]
pub(super) struct SourceDialect;
impl Dialect for SourceDialect {
    fn dialect(&self) -> std::any::TypeId {
        std::any::TypeId::of::<PostgreSqlDialect>()
    }
    fn is_identifier_start(&self, ch: char) -> bool {
        PostgreSqlDialect {}.is_identifier_start(ch)
    }
    fn is_identifier_part(&self, ch: char) -> bool {
        PostgreSqlDialect {}.is_identifier_part(ch)
    }
    fn is_delimited_identifier_start(&self, ch: char) -> bool {
        PostgreSqlDialect {}.is_delimited_identifier_start(ch)
    }
    fn is_custom_operator_part(&self, ch: char) -> bool {
        PostgreSqlDialect {}.is_custom_operator_part(ch)
    }
    fn supports_nested_comments(&self) -> bool {
        true
    }
    fn supports_string_escape_constant(&self) -> bool {
        true
    }
    fn supports_numeric_literal_underscores(&self) -> bool {
        true
    }
    fn supports_geometric_types(&self) -> bool {
        true
    }
}

pub(super) fn prepare(tokens: &mut Vec<TokenWithSpan>) {
    let mut output = Vec::with_capacity(tokens.len());
    let mut index = 0;
    while index < tokens.len() {
        let parts = &tokens[index..];
        let literal = match parts {
            [prefix, ampersand, literal, ..]
                if matches!(&prefix.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("u"))
                    && ampersand.token == Token::Ampersand
                    && prefix.span.end == ampersand.span.start
                    && ampersand.span.end == literal.span.start =>
            {
                if let Token::SingleQuotedString(value) = &literal.token {
                    Some((prefix, literal, value))
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some((prefix, literal, value)) = literal {
            // The plain-string tokenizer already unescapes doubled quotes. Restore
            // that encoding for the shared Unicode decoder, which also owns quotes.
            let decoded = super::unicode::decode_unicode_string(
                &value.replace('\'', "''"),
                escape_after(&tokens[index + 3..]),
            );
            let token = match decoded {
                Some(value) if !value.contains('\0') => Token::UnicodeStringLiteral(value),
                // Preserve a located invalid token so recovery can keep neighbors.
                _ => Token::Char('\0'),
            };
            output.push(TokenWithSpan {
                token,
                span: sqlparser::tokenizer::Span {
                    start: prefix.span.start,
                    end: literal.span.end,
                },
            });
            index += 3;
        } else {
            output.push(tokens[index].clone());
            index += 1;
        }
    }
    *tokens = output;
}

fn escape_after(tokens: &[TokenWithSpan]) -> char {
    let mut tokens = tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)));
    if !matches!(tokens.next().map(|t| &t.token), Some(Token::Word(word)) if word.quote_style.is_none() && word.keyword == Keyword::UESCAPE)
    {
        return '\\';
    }
    let Some(Token::SingleQuotedString(value)) = tokens.next().map(|t| &t.token) else {
        return '\\';
    };
    let mut chars = value.chars();
    match (chars.next(), chars.next()) {
        (Some(escape), None) => escape,
        _ => '\\',
    }
}

#[cfg(test)]
mod tests;
