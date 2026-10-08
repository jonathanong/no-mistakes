//! Recover the declaration boundary without mistaking query aliases for bodies.
use super::*;
mod end_alias;

pub(super) fn end(parser: &Parser<'_>, start: usize) -> usize {
    let mut depth: usize = 0;
    let mut cases = Vec::new();
    let mut parentheses: usize = 0;
    let mut declaration = false;
    let mut index = start;
    loop {
        let token = parser.token_at(index);
        if token.token == Token::EOF {
            return index;
        }
        match token.token {
            Token::LParen => parentheses += 1,
            Token::RParen => parentheses = parentheses.saturating_sub(1),
            _ => {}
        }
        if keyword(&token.token, Keyword::CREATE) && child_boundary(parser, index) {
            declaration = declaration_start(parser, index);
        } else if token.token == Token::SemiColon {
            declaration = false;
        }
        // BEGIN and ATOMIC are unreserved PostgreSQL identifiers. Only the
        // initial body, child boundary, and nested declaration introduce blocks.
        let block = keyword(&token.token, Keyword::BEGIN)
            && parentheses == 0
            && (depth == 0
                || (declaration || child_boundary(parser, index))
                    && keyword(&parser.token_at(next(parser, index)).token, Keyword::ATOMIC));
        if block {
            declaration = false;
            depth += 1;
        } else if keyword(&token.token, Keyword::CASE) && case_expression(parser, index) {
            cases.push(parentheses);
        } else if keyword(&token.token, Keyword::END) && !label(parser, index) {
            if cases.last() == Some(&parentheses) && !child_boundary(parser, index) {
                cases.pop();
            } else if !end_alias::continues_query(parser, index) {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return index;
                }
            }
        }
        index += 1;
    }
}

fn declaration_start(parser: &Parser<'_>, index: usize) -> bool {
    let mut index = next(parser, index);
    if keyword(&parser.token_at(index).token, Keyword::OR) {
        index = next(parser, index);
        if !keyword(&parser.token_at(index).token, Keyword::REPLACE) {
            return false;
        }
        index = next(parser, index);
    }
    [Keyword::FUNCTION, Keyword::PROCEDURE]
        .iter()
        .any(|value| keyword(&parser.token_at(index).token, *value))
}

fn next(parser: &Parser<'_>, index: usize) -> usize {
    let mut index = index + 1;
    while matches!(parser.token_at(index).token, Token::Whitespace(_)) {
        index += 1;
    }
    index
}

fn child_boundary(parser: &Parser<'_>, index: usize) -> bool {
    let before = previous(parser, index);
    parser.token_at(before).token == Token::SemiColon
        || keyword(&parser.token_at(before).token, Keyword::ATOMIC)
            && keyword(
                &parser.token_at(previous(parser, before)).token,
                Keyword::BEGIN,
            )
}

fn label(parser: &Parser<'_>, index: usize) -> bool {
    let token = parser.token_at(previous(parser, index));
    // ColLabel accepts reserved keywords after AS and qualified field access.
    token.token == Token::Period || keyword(&token.token, Keyword::AS)
}

fn case_expression(parser: &Parser<'_>, index: usize) -> bool {
    if label(parser, index) {
        return false;
    }
    if case_function(parser, index) {
        return true;
    }
    // A bare CASE label can end a projection. A CASE expression always has a
    // condition or WHEN next, never one of these projection/statement boundaries.
    match &parser.token_at(next(parser, index)).token {
        Token::SemiColon | Token::Comma | Token::RParen | Token::EOF => false,
        Token::Word(word) => ![
            Keyword::FROM,
            Keyword::INTO,
            Keyword::WHERE,
            Keyword::GROUP,
            Keyword::HAVING,
            Keyword::WINDOW,
            Keyword::ORDER,
            Keyword::LIMIT,
            Keyword::OFFSET,
            Keyword::FETCH,
            Keyword::UNION,
            Keyword::INTERSECT,
            Keyword::EXCEPT,
            Keyword::FOR,
            Keyword::JOIN,
            Keyword::INNER,
            Keyword::LEFT,
            Keyword::RIGHT,
            Keyword::FULL,
            Keyword::CROSS,
            Keyword::ON,
            Keyword::USING,
        ]
        .contains(&word.keyword),
        _ => true,
    }
}

fn case_function(parser: &Parser<'_>, index: usize) -> bool {
    let function = next(parser, index);
    if ![
        Keyword::LEFT,
        Keyword::RIGHT,
        Keyword::INNER,
        Keyword::FULL,
        Keyword::CROSS,
        Keyword::JOIN,
    ]
    .iter()
    .any(|value| keyword(&parser.token_at(function).token, *value))
        || parser.token_at(next(parser, function)).token != Token::LParen
    {
        return false;
    }
    let previous = parser.token_at(previous(parser, index));
    // A word/closed table factor before CASE makes it a bare alias, as in
    // FROM target CASE JOIN (SELECT ...), rather than a new CASE expression.
    match &previous.token {
        Token::Word(word) => {
            word.quote_style.is_none()
                && [
                    Keyword::SELECT,
                    Keyword::THEN,
                    Keyword::ELSE,
                    Keyword::WHEN,
                    Keyword::WHERE,
                    Keyword::HAVING,
                    Keyword::RETURNING,
                    Keyword::ON,
                    Keyword::BY,
                    Keyword::AND,
                    Keyword::OR,
                    Keyword::NOT,
                    Keyword::DISTINCT,
                    Keyword::ALL,
                    Keyword::BETWEEN,
                    Keyword::FROM,
                    Keyword::ZONE,
                    Keyword::LIMIT,
                    Keyword::OFFSET,
                    Keyword::DEFAULT,
                    Keyword::RETURN,
                    Keyword::IN,
                    Keyword::BOTH,
                    Keyword::LEADING,
                    Keyword::TRAILING,
                    Keyword::PLACING,
                    Keyword::SIMILAR,
                    Keyword::ESCAPE,
                    Keyword::FIRST,
                    Keyword::NEXT,
                ]
                .contains(&word.keyword)
        }
        _ => previous.token != Token::RParen,
    }
}

fn previous(parser: &Parser<'_>, index: usize) -> usize {
    let mut index = index.saturating_sub(1);
    while index > 0 && matches!(parser.token_at(index).token, Token::Whitespace(_)) {
        index -= 1;
    }
    index
}
