use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};

/// sqlparser 0.63 can consume a parenthesized set arm's trailing LIMIT (or
/// reject the closing parenthesis) while parsing TABLE. Its SELECT equivalent
/// keeps the clause on the derived query without changing source positions.
pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    let mut depth = 0usize;
    let mut previous = 0usize;
    let mut before_previous = None;
    let mut rewrite_at = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if matches!(token.token, Token::Whitespace(_)) {
            continue;
        }
        if matches!(token.token, Token::Word(ref word) if word.quote_style.is_none() && word.keyword == Keyword::TABLE)
            && depth > 0
            && set_operator_before(tokens, previous, before_previous)
            && followed_by_limit_or_close(tokens, index)
        {
            rewrite_at.push(index);
        }
        match token.token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
        before_previous = Some(previous);
        previous = index;
    }
    if rewrite_at.is_empty() {
        return;
    }
    let mut rewrite_at = rewrite_at.into_iter().peekable();
    let mut result = Vec::with_capacity(tokens.len());
    for (index, token) in tokens.drain(..).enumerate() {
        if rewrite_at.peek() == Some(&index) {
            rewrite_at.next();
            let span = token.span;
            result.push(TokenWithSpan::new(word("SELECT", Keyword::SELECT), span));
            result.push(TokenWithSpan::new(Token::Mul, span));
            result.push(TokenWithSpan::new(word("FROM", Keyword::FROM), span));
        } else {
            result.push(token);
        }
    }
    *tokens = result;
}

fn set_operator_before(
    tokens: &[TokenWithSpan],
    previous: usize,
    before_previous: Option<usize>,
) -> bool {
    let keyword = |index: usize| match &tokens[index].token {
        Token::Word(word) if word.quote_style.is_none() => Some(word.keyword),
        _ => None,
    };
    if is_set_operator(keyword(previous)) {
        return true;
    }
    matches!(keyword(previous), Some(Keyword::ALL | Keyword::DISTINCT))
        && before_previous.is_some_and(|index| is_set_operator(keyword(index)))
}

fn is_set_operator(keyword: Option<Keyword>) -> bool {
    matches!(
        keyword,
        Some(Keyword::UNION | Keyword::INTERSECT | Keyword::EXCEPT)
    )
}

fn followed_by_limit_or_close(tokens: &[TokenWithSpan], at: usize) -> bool {
    let mut after = tokens[at + 1..]
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .map(|token| &token.token);
    if !matches!(after.next(), Some(Token::Word(_))) {
        return false;
    }
    matches!(
        after.next(),
        Some(Token::RParen)
            | Some(Token::Word(Word {
                keyword: Keyword::LIMIT,
                ..
            }))
    )
}

fn word(value: &str, keyword: Keyword) -> Token {
    Token::Word(Word {
        value: value.to_owned(),
        quote_style: None,
        keyword,
    })
}
