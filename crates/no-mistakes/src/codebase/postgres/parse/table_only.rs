use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Whitespace, Word};

/// sqlparser 0.63 rejects PostgreSQL's `TABLE ONLY name`; in INSERT it also
/// mistakes TABLE for a target alias. Rewrite only the parser's token clone,
/// preserving source locations and the original located token stream.
pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    let words: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token.token, Token::Whitespace(_)))
        .map(|(index, _)| index)
        .collect();
    let mut insert_sources = Vec::new();
    for (position, &index) in words.iter().enumerate() {
        let Token::Word(table) = &tokens[index].token else {
            continue;
        };
        if table.keyword != Keyword::TABLE {
            continue;
        }
        let insert_source = insert_source_context(tokens, &words, position);
        if !insert_source
            && !query_context(
                position
                    .checked_sub(1)
                    .map(|before| &tokens[words[before]].token),
            )
        {
            continue;
        }
        let Some(&only_index) = words.get(position + 1) else {
            continue;
        };
        if matches!(&tokens[only_index].token, Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::ONLY)
        {
            tokens[only_index].token = Token::Whitespace(Whitespace::Space);
            if insert_source {
                // sqlparser's INSERT AST has no OVERRIDING field. The identity
                // mode does not change which source relation the query reads.
                if position >= 3 && keyword(&tokens[words[position - 3]].token, "OVERRIDING") {
                    for &override_index in &words[position - 3..position] {
                        tokens[override_index].token = Token::Whitespace(Whitespace::Space);
                    }
                }
                insert_sources.push(index);
            }
        }
    }
    if insert_sources.is_empty() {
        return;
    }
    let mut inserts = insert_sources.into_iter().peekable();
    let mut result = Vec::with_capacity(tokens.len() + 2 * inserts.len());
    for (index, mut token) in tokens.drain(..).enumerate() {
        if inserts.peek() == Some(&index) {
            inserts.next();
            let span = token.span;
            token.token = word_token("SELECT", Keyword::SELECT);
            result.push(token);
            result.push(TokenWithSpan::new(Token::Mul, span));
            result.push(TokenWithSpan::new(word_token("FROM", Keyword::FROM), span));
        } else {
            result.push(token);
        }
    }
    *tokens = result;
}

fn word_token(value: &str, keyword: Keyword) -> Token {
    Token::Word(Word {
        value: value.to_owned(),
        quote_style: None,
        keyword,
    })
}

/// The INSERT target can have an alias, column list, and identity override
/// before its TABLE query source. Match that prefix without changing DDL TABLE.
fn insert_source_context(tokens: &[TokenWithSpan], words: &[usize], table_at: usize) -> bool {
    let Some(insert_at) = words[..table_at]
        .iter()
        .rposition(|&index| keyword(&tokens[index].token, "INSERT"))
    else {
        return false;
    };
    if words[insert_at + 1..table_at]
        .iter()
        .any(|&index| matches!(tokens[index].token, Token::SemiColon))
    {
        return false;
    }
    let before = &words[..table_at];
    let mut cursor = insert_at + 1;
    if !take_keyword(tokens, before, &mut cursor, "INTO") {
        return false;
    }
    if !take_name(tokens, before, &mut cursor) {
        return false;
    }
    while matches!(at(tokens, before, cursor), Some(Token::Period)) {
        cursor += 1;
        if !take_name(tokens, before, &mut cursor) {
            return false;
        }
    }
    if take_keyword(tokens, before, &mut cursor, "AS") && !take_name(tokens, before, &mut cursor) {
        return false;
    }
    if matches!(at(tokens, before, cursor), Some(Token::LParen)) {
        let mut depth = 0;
        while let Some(token) = at(tokens, before, cursor) {
            cursor += 1;
            match token {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {}
            }
        }
        if depth != 0 {
            return false;
        }
    }
    if take_keyword(tokens, before, &mut cursor, "OVERRIDING")
        && (!(take_keyword(tokens, before, &mut cursor, "SYSTEM")
            || take_keyword(tokens, before, &mut cursor, "USER"))
            || !take_keyword(tokens, before, &mut cursor, "VALUE"))
    {
        return false;
    }
    cursor == table_at
}

fn at<'a>(tokens: &'a [TokenWithSpan], words: &[usize], cursor: usize) -> Option<&'a Token> {
    words.get(cursor).map(|&index| &tokens[index].token)
}

fn take_keyword(
    tokens: &[TokenWithSpan],
    words: &[usize],
    cursor: &mut usize,
    value: &str,
) -> bool {
    if at(tokens, words, *cursor).is_some_and(|token| keyword(token, value)) {
        *cursor += 1;
        true
    } else {
        false
    }
}

fn take_name(tokens: &[TokenWithSpan], words: &[usize], cursor: &mut usize) -> bool {
    if matches!(at(tokens, words, *cursor), Some(Token::Word(_))) {
        *cursor += 1;
        true
    } else {
        false
    }
}

fn keyword(token: &Token, value: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(value))
}

fn query_context(previous: Option<&Token>) -> bool {
    match previous {
        None | Some(Token::SemiColon | Token::LParen | Token::RParen) => true,
        Some(Token::Word(word)) => matches!(
            word.value.to_ascii_uppercase().as_str(),
            "ALL" | "DISTINCT" | "UNION" | "INTERSECT" | "EXCEPT" | "AS"
        ),
        _ => false,
    }
}
