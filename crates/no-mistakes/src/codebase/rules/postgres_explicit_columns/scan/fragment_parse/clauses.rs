use super::{is_kw, next_code, read_optional_alias, skip_paren_group, word_value, FragmentContext};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

#[derive(Clone, Copy, Default)]
pub(super) struct OpenClause {
    pub(super) select: bool,
    pub(super) returning: bool,
    pub(super) in_exists: bool,
    pub(super) expects_expr: bool,
}

pub(super) fn open_clause_at_end(tokens: &[TokenWithSpan]) -> OpenClause {
    let mut stack: Vec<(bool, bool, bool)> = Vec::new();
    let mut select = false;
    let mut returning = false;
    let mut in_exists = false;
    let mut prev_exists = false;
    let mut last_code: Option<(usize, &TokenWithSpan)> = None;
    for (i, token) in tokens.iter().enumerate() {
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
        last_code = Some((i, token));
    }
    let expects_expr =
        last_code.is_some_and(|(idx, tok)| super::readers::is_bare_star_prev(tokens, idx, tok));
    OpenClause {
        select,
        returning,
        in_exists,
        expects_expr,
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

pub(super) fn consume_table_function(
    tokens: &[TokenWithSpan],
    index: usize,
    ctx: &mut FragmentContext,
) -> Option<usize> {
    let (i, tok) = next_code(tokens, index)?;
    if is_kw(tok, Keyword::ONLY) {
        return None;
    }
    word_value(tok)?;
    let mut next_i = i + 1;
    if let Some((dot_i, dot)) = next_code(tokens, next_i) {
        if matches!(dot.token, Token::Period) {
            if let Some((part_i, part)) = next_code(tokens, dot_i + 1) {
                if word_value(part).is_some() {
                    next_i = part_i + 1;
                }
            }
        }
    }
    let (paren_i, paren_tok) = next_code(tokens, next_i)?;
    if !matches!(paren_tok.token, Token::LParen) {
        return None;
    }
    let mut after_args = skip_paren_group(tokens, paren_i);
    if let Some((with_i, with_tok)) = next_code(tokens, after_args) {
        if is_kw(with_tok, Keyword::WITH) {
            if let Some((ord_i, ord_tok)) = next_code(tokens, with_i + 1) {
                if is_kw(ord_tok, Keyword::ORDINALITY) {
                    after_args = ord_i + 1;
                }
            }
        }
    }
    let (mut end, alias) = read_optional_alias(tokens, after_args);
    if let Some((cols_i, cols_tok)) = next_code(tokens, end) {
        if matches!(cols_tok.token, Token::LParen) {
            end = skip_paren_group(tokens, cols_i);
        }
    }
    ctx.has_derived_from = true;
    if let Some(name) = alias {
        ctx.derived_aliases.push(name);
    }
    Some(end)
}
