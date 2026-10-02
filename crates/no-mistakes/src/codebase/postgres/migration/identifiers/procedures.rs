use crate::codebase::postgres::parse::unicode::decode_unicode_string;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn identifier(tokens: &[TokenWithSpan], mut at: usize) -> Option<(String, usize)> {
    let mut name = component(tokens, at)?;
    loop {
        at = name.1;
        if !matches!(
            tokens.get(at).map(|token| &token.token),
            Some(Token::Period)
        ) {
            return Some(name);
        }
        name = component(tokens, at + 1)?;
    }
}

fn component(tokens: &[TokenWithSpan], at: usize) -> Option<(String, usize)> {
    let Token::Word(first) = &tokens.get(at)?.token else {
        return None;
    };
    if !first.value.eq_ignore_ascii_case("U")
        || first.quote_style.is_some()
        || !matches!(
            tokens.get(at + 1).map(|token| &token.token),
            Some(Token::Ampersand)
        )
    {
        return Some((first.value.clone(), at + 1));
    }
    let Token::Word(value) = &tokens.get(at + 2)?.token else {
        return None;
    };
    if value.quote_style != Some('"')
        || tokens[at].span.end != tokens[at + 1].span.start
        || tokens[at + 1].span.end != tokens[at + 2].span.start
    {
        return None;
    }
    let mut end = at + 3;
    let escape = if super::locate::word(tokens.get(end), "UESCAPE") {
        let Some(Token::SingleQuotedString(escape)) = tokens.get(end + 1).map(|token| &token.token)
        else {
            return None;
        };
        let mut chars = escape.chars();
        let escape = chars.next()?;
        if chars.next().is_some() {
            return None;
        }
        end += 2;
        escape
    } else {
        '\\'
    };
    // The shared decoder handles SQL string quotes; protect identifier apostrophes.
    let name = decode_unicode_string(&value.value.replace('\'', "''"), escape)?;
    Some((name, end))
}
