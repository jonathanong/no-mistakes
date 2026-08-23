use super::{keyword_of, next_non_ws};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::Token;

pub(super) fn rewrite_drop_index_concurrently(tokens: &mut Vec<Token>) {
    let mut index = 0;
    while index < tokens.len() {
        if keyword_of(&tokens[index]) == Some(Keyword::DROP) {
            if let Some(concurrent) = drop_index_concurrently_at(tokens, index) {
                tokens.remove(concurrent);
                continue;
            }
        }
        index += 1;
    }
}

fn drop_index_concurrently_at(tokens: &[Token], drop_at: usize) -> Option<usize> {
    let index_at = next_non_ws(tokens, drop_at + 1)?;
    if keyword_of(&tokens[index_at]) != Some(Keyword::INDEX) {
        return None;
    }
    let concurrent_at = next_non_ws(tokens, index_at + 1)?;
    (keyword_of(&tokens[concurrent_at]) == Some(Keyword::CONCURRENTLY)).then_some(concurrent_at)
}

pub(super) fn rewrite_chr_calls(tokens: &mut Vec<Token>) {
    let mut index = 0;
    while index < tokens.len() {
        if let Some((end, value)) = chr_call_at(tokens, index) {
            tokens.splice(index..=end, [Token::SingleQuotedString(value)]);
        }
        index += 1;
    }
}

fn chr_call_at(tokens: &[Token], start: usize) -> Option<(usize, String)> {
    if !is_chr_ident(&tokens[start]) {
        return None;
    }
    let open = next_non_ws(tokens, start + 1)?;
    if !matches!(tokens.get(open)?, Token::LParen) {
        return None;
    }
    let number_at = next_non_ws(tokens, open + 1)?;
    let Token::Number(raw, _) = tokens.get(number_at)? else {
        return None;
    };
    let code = raw.parse::<u32>().ok()?;
    let value = char::from_u32(code)?.to_string();
    let close = next_non_ws(tokens, number_at + 1)?;
    matches!(tokens.get(close)?, Token::RParen).then_some((close, value))
}

fn is_chr_ident(token: &Token) -> bool {
    matches!(token, Token::Word(word) if word.value.eq_ignore_ascii_case("chr"))
}

#[cfg(test)]
#[path = "rewrite/tests.rs"]
mod tests;
