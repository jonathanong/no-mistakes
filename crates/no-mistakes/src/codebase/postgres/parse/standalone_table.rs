use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};

/// sqlparser 0.63 accepts TABLE as a query arm but not as a top-level SQL
/// statement. Rewrite only statement-leading TABLE to its SELECT equivalent.
pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    if !has_standalone_table(tokens) {
        return;
    }
    let mut rewrite_at = Vec::new();
    let mut at_statement_start = true;
    for (index, token) in tokens.iter().enumerate() {
        match &token.token {
            Token::Whitespace(_) => {}
            Token::SemiColon => at_statement_start = true,
            Token::Word(word)
                if at_statement_start
                    && word.quote_style.is_none()
                    && word.keyword == Keyword::TABLE
                    && starts_table_query(tokens, index) =>
            {
                rewrite_at.push(index);
                at_statement_start = false;
            }
            _ => at_statement_start = false,
        }
    }
    let mut rewrite_at = rewrite_at.into_iter().peekable();
    let mut result = Vec::with_capacity(tokens.len());
    let mut at_statement_start = true;
    for (index, token) in tokens.drain(..).enumerate() {
        match &token.token {
            Token::Whitespace(_) => result.push(token),
            Token::SemiColon => {
                result.push(token);
                at_statement_start = true;
            }
            Token::Word(word)
                if at_statement_start
                    && word.quote_style.is_none()
                    && word.keyword == Keyword::TABLE
                    && rewrite_at.peek() == Some(&index) =>
            {
                rewrite_at.next();
                let span = token.span;
                result.push(TokenWithSpan::new(
                    word_token("SELECT", Keyword::SELECT),
                    span,
                ));
                result.push(TokenWithSpan::new(Token::Mul, span));
                result.push(TokenWithSpan::new(word_token("FROM", Keyword::FROM), span));
                at_statement_start = false;
            }
            _ => {
                result.push(token);
                at_statement_start = false;
            }
        }
    }
    *tokens = result;
}

fn has_standalone_table(tokens: &[TokenWithSpan]) -> bool {
    let mut at_statement_start = true;
    for (index, token) in tokens.iter().enumerate() {
        match &token.token {
            Token::Whitespace(_) => {}
            Token::SemiColon => at_statement_start = true,
            Token::Word(word)
                if at_statement_start
                    && word.quote_style.is_none()
                    && word.keyword == Keyword::TABLE
                    && starts_table_query(tokens, index) =>
            {
                return true;
            }
            _ => at_statement_start = false,
        }
    }
    false
}

fn starts_table_query(tokens: &[TokenWithSpan], at: usize) -> bool {
    let mut after = tokens[at + 1..]
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)));
    // TABLE takes a relation name, optionally qualified by a schema. A SELECT-only
    // clause after that name must remain a parse error, not become valid SELECT.
    let Some(Token::Word(_)) = after.next().map(|token| &token.token) else {
        return false;
    };
    let mut next = after.next().map(|token| &token.token);
    if matches!(next, Some(Token::Period)) {
        if !matches!(after.next().map(|token| &token.token), Some(Token::Word(_))) {
            return false;
        }
        next = after.next().map(|token| &token.token);
    }
    matches!(
        next,
        None | Some(Token::SemiColon)
            | Some(Token::RParen)
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
}

fn word_token(value: &str, keyword: Keyword) -> Token {
    Token::Word(Word {
        value: value.to_owned(),
        quote_style: None,
        keyword,
    })
}
