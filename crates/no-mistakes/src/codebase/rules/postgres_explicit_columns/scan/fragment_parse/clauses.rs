use super::is_kw;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn open_clause_at_end(tokens: &[TokenWithSpan]) -> (bool, bool, bool) {
    let mut stack: Vec<(bool, bool, bool)> = Vec::new();
    let mut select = false;
    let mut returning = false;
    let mut in_exists = false;
    let mut prev_exists = false;
    for token in tokens {
        if matches!(token.token, Token::Whitespace(_)) {
            continue;
        }
        match &token.token {
            Token::LParen => {
                stack.push((select, returning, in_exists));
                select = false;
                returning = false;
                in_exists = prev_exists;
            }
            Token::RParen => {
                if let Some((ps, pr, pe)) = stack.pop() {
                    select = ps;
                    returning = pr;
                    in_exists = pe;
                }
            }
            _ if is_kw(token, Keyword::SELECT) => {
                select = true;
                returning = false;
            }
            _ if is_kw(token, Keyword::RETURNING) => {
                returning = true;
                select = false;
            }
            Token::Word(w)
                if matches!(
                    w.keyword,
                    Keyword::FROM
                        | Keyword::WHERE
                        | Keyword::INTO
                        | Keyword::SET
                        | Keyword::VALUES
                        | Keyword::GROUP
                        | Keyword::ORDER
                        | Keyword::LIMIT
                        | Keyword::HAVING
                        | Keyword::UNION
                        | Keyword::INTERSECT
                        | Keyword::EXCEPT
                ) =>
            {
                select = false;
                returning = false;
            }
            _ => {}
        }
        prev_exists = is_kw(token, Keyword::EXISTS);
    }
    (select, returning, in_exists)
}

pub(super) fn update_paren_depth(token: &TokenWithSpan, depth: &mut usize) -> bool {
    match &token.token {
        Token::LParen => {
            *depth += 1;
            true
        }
        Token::RParen => {
            *depth = depth.saturating_sub(1);
            true
        }
        _ => false,
    }
}

pub(super) fn ends_from_list(token: &TokenWithSpan) -> bool {
    matches!(
        &token.token,
        Token::Word(w) if matches!(
            w.keyword,
            Keyword::WHERE
                | Keyword::GROUP
                | Keyword::HAVING
                | Keyword::ORDER
                | Keyword::LIMIT
                | Keyword::OFFSET
                | Keyword::FETCH
                | Keyword::UNION
                | Keyword::INTERSECT
                | Keyword::EXCEPT
                | Keyword::RETURNING
                | Keyword::SET
                | Keyword::VALUES
                | Keyword::SELECT
                | Keyword::DELETE
                | Keyword::WINDOW
        )
    )
}

pub(super) fn is_implicit_alias_word(token: &TokenWithSpan) -> bool {
    let Token::Word(word) = &token.token else {
        return false;
    };
    word.quote_style.is_some()
        || (!sqlparser::keywords::RESERVED_FOR_TABLE_ALIAS.contains(&word.keyword)
            && !ends_from_list(token)
            && !matches!(
                word.keyword,
                Keyword::AS
                    | Keyword::ON
                    | Keyword::USING
                    | Keyword::JOIN
                    | Keyword::INNER
                    | Keyword::LEFT
                    | Keyword::RIGHT
                    | Keyword::FULL
                    | Keyword::OUTER
                    | Keyword::CROSS
                    | Keyword::NATURAL
                    | Keyword::LATERAL
                    | Keyword::FOR
                    | Keyword::DO
                    | Keyword::CONFLICT
                    | Keyword::INTO
                    | Keyword::UPDATE
                    | Keyword::INSERT
                    | Keyword::FROM
            ))
}
