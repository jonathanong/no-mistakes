use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};

/// sqlparser 0.63 rejects top-level or scalar TABLE queries, and an unqualified
/// middle TABLE arm consumes the next set operator. Rewrite those arms to their
/// SELECT equivalent while preserving the original relation tokens and spans.
pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    if !has_rewrite(tokens) {
        return;
    }
    let mut rewrite_at = Vec::new();
    let mut omit_star_at = Vec::new();
    let mut leading = Leading::Start;
    let (mut previous, mut before_previous) = (None, None);
    for (index, token) in tokens.iter().enumerate() {
        let at_start = leading_table(&mut leading, &token.token);
        if is_table(&token.token) {
            if let Some(query) = table_query(tokens, index)
                .filter(|query| should_rewrite(at_start, query, previous, before_previous))
            {
                rewrite_at.push(index);
                omit_star_at.extend(query.omit_star_at);
            }
        }
        advance_previous(&token.token, &mut previous, &mut before_previous);
    }
    let mut rewrite_at = rewrite_at.into_iter().peekable();
    let mut omit_star_at = omit_star_at.into_iter().peekable();
    let mut result = Vec::with_capacity(tokens.len());
    for (index, token) in tokens.drain(..).enumerate() {
        if omit_star_at.peek() == Some(&index) {
            omit_star_at.next();
            continue;
        }
        if rewrite_at.peek() == Some(&index) {
            rewrite_at.next();
            let span = token.span;
            result.push(TokenWithSpan::new(
                word_token("SELECT", Keyword::SELECT),
                span,
            ));
            result.push(TokenWithSpan::new(Token::Mul, span));
            result.push(TokenWithSpan::new(word_token("FROM", Keyword::FROM), span));
        } else {
            result.push(token);
        }
    }
    *tokens = result;
}

fn has_rewrite(tokens: &[TokenWithSpan]) -> bool {
    let mut leading = Leading::Start;
    let (mut previous, mut before_previous) = (None, None);
    for (index, token) in tokens.iter().enumerate() {
        let at_start = leading_table(&mut leading, &token.token);
        if is_table(&token.token) {
            if let Some(query) = table_query(tokens, index) {
                if should_rewrite(at_start, &query, previous, before_previous) {
                    return true;
                }
            }
        }
        advance_previous(&token.token, &mut previous, &mut before_previous);
    }
    false
}

fn should_rewrite(
    at_start: bool,
    query: &TableQuery,
    previous: Option<&Token>,
    before_previous: Option<&Token>,
) -> bool {
    at_start
        || !query.qualified
            && query.followed_by_set_op
            && preceded_by_set_op(previous, before_previous)
}

fn is_table(token: &Token) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::TABLE)
}

fn preceded_by_set_op(previous: Option<&Token>, before_previous: Option<&Token>) -> bool {
    is_set_op(previous)
        || matches!(previous, Some(Token::Word(word)) if matches!(word.keyword, Keyword::ALL | Keyword::DISTINCT))
            && is_set_op(before_previous)
}

fn is_set_op(token: Option<&Token>) -> bool {
    matches!(token, Some(Token::Word(word)) if matches!(word.keyword, Keyword::UNION | Keyword::INTERSECT | Keyword::EXCEPT))
}

fn advance_previous<'a>(
    token: &'a Token,
    previous: &mut Option<&'a Token>,
    before_previous: &mut Option<&'a Token>,
) {
    if !matches!(token, Token::Whitespace(_)) {
        *before_previous = *previous;
        *previous = Some(token);
    }
}

#[derive(Clone, Copy)]
enum Leading {
    Start,
    Explain,
    Body,
}

fn leading_table(state: &mut Leading, token: &Token) -> bool {
    match token {
        Token::Whitespace(_) => false,
        Token::SemiColon | Token::LParen => {
            // Parentheses may begin a scalar TABLE query, including inside a view.
            *state = Leading::Start;
            false
        }
        Token::Word(word)
            if matches!(*state, Leading::Start)
                && word.quote_style.is_none()
                && word.keyword == Keyword::EXPLAIN =>
        {
            *state = Leading::Explain;
            false
        }
        Token::Word(word)
            if matches!(*state, Leading::Explain)
                && word.quote_style.is_none()
                && matches!(word.keyword, Keyword::ANALYZE | Keyword::VERBOSE) =>
        {
            false
        }
        Token::Word(word)
            if !matches!(*state, Leading::Body)
                && word.quote_style.is_none()
                && word.keyword == Keyword::TABLE =>
        {
            *state = Leading::Body;
            true
        }
        _ => {
            *state = Leading::Body;
            false
        }
    }
}

/// Valid TABLE prefix, with the optional inheritance `*` token to omit from
/// sqlparser's equivalent SELECT form. None means the original SQL must parse as-is.
struct TableQuery {
    omit_star_at: Option<usize>,
    followed_by_set_op: bool,
    qualified: bool,
}

fn table_query(tokens: &[TokenWithSpan], at: usize) -> Option<TableQuery> {
    let mut after = tokens[at + 1..]
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token.token, Token::Whitespace(_)))
        .map(|(index, token)| (at + index + 1, &token.token));
    // TABLE takes a relation name, optionally qualified by a schema. A SELECT-only
    // clause after that name must remain a parse error, not become valid SELECT.
    let Some((_, Token::Word(_))) = after.next() else {
        return None;
    };
    let mut next = after.next();
    let qualified = matches!(next, Some((_, Token::Period)));
    if qualified {
        if !matches!(after.next(), Some((_, Token::Word(_)))) {
            return None;
        }
        next = after.next();
    }
    let star_at = if let Some((index, Token::Mul)) = next {
        next = after.next();
        Some(index)
    } else {
        None
    };
    let followed_by_set_op = is_set_op(next.map(|(_, token)| token));
    matches!(
        next.map(|(_, token)| token),
        None | Some(Token::SemiColon | Token::RParen)
            | Some(Token::Word(Word {
                keyword: Keyword::UNION
                    | Keyword::INTERSECT
                    | Keyword::EXCEPT
                    | Keyword::ORDER
                    | Keyword::LIMIT
                    | Keyword::OFFSET
                    | Keyword::FETCH
                    | Keyword::FOR,
                ..
            }))
    )
    .then_some(TableQuery {
        omit_star_at: star_at,
        followed_by_set_op,
        qualified,
    })
}

fn word_token(value: &str, keyword: Keyword) -> Token {
    Token::Word(Word {
        value: value.to_owned(),
        quote_style: None,
        keyword,
    })
}
