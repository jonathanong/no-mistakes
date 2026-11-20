use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn owns_terminator(original: &[TokenWithSpan]) -> bool {
    super::super::super::copy_data::is_copy_stdin_tokens(original)
}

/// sqlparser 0.64 keeps an exact `\.` line, so peel it from the next statement.
pub(super) fn pending_body(
    pending: &mut bool,
    original: &mut Option<&[TokenWithSpan]>,
    mut chunk: Vec<Token>,
) -> Option<Vec<Token>> {
    if !*pending {
        return Some(chunk);
    }
    let Some(rest) = (*original).and_then(without_terminator) else {
        return Some(chunk);
    };
    *pending = false;
    if rest
        .iter()
        .all(|token| matches!(token.token, Token::Whitespace(_)))
    {
        return None;
    }
    *original = Some(rest);
    peel_chunk(&mut chunk);
    Some(chunk)
}

fn without_terminator(tokens: &[TokenWithSpan]) -> Option<&[TokenWithSpan]> {
    let mut index = skip_spans(tokens, 0);
    if tokens
        .get(index)
        .is_none_or(|token| token.token != Token::Backslash)
    {
        return None;
    }
    index = skip_spans(tokens, index + 1);
    if tokens
        .get(index)
        .is_none_or(|token| token.token != Token::Period)
    {
        return None;
    }
    Some(&tokens[index + 1..])
}

fn peel_chunk(chunk: &mut Vec<Token>) {
    let mut index = skip_tokens(chunk, 0);
    if chunk
        .get(index)
        .is_none_or(|token| *token != Token::Backslash)
    {
        return;
    }
    index = skip_tokens(chunk, index + 1);
    if chunk.get(index).is_none_or(|token| *token != Token::Period) {
        return;
    }
    chunk.drain(..index + 1);
}

fn skip_spans(tokens: &[TokenWithSpan], mut index: usize) -> usize {
    while tokens
        .get(index)
        .is_some_and(|token| matches!(token.token, Token::Whitespace(_)))
    {
        index += 1;
    }
    index
}

fn skip_tokens(tokens: &[Token], mut index: usize) -> usize {
    while tokens
        .get(index)
        .is_some_and(|token| matches!(token, Token::Whitespace(_)))
    {
        index += 1;
    }
    index
}
