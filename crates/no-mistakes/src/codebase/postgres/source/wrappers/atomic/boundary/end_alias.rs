use super::*;

/// Bare END is a PostgreSQL query label, even where the native parser rejects
/// it. Continuing query tokens preserve that child's full atomic ownership.
pub(super) fn continues_query(parser: &Parser<'_>, index: usize) -> bool {
    if child_boundary(parser, index) {
        return false;
    }
    match &parser.token_at(next(parser, index)).token {
        Token::Comma | Token::RParen => true,
        Token::Word(word) if word.quote_style.is_none() => [
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
        _ => false,
    }
}
