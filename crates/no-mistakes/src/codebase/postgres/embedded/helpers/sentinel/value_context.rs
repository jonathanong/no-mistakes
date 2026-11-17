use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, Tokenizer};

pub(super) fn recognizes(prefix: &str, tail: char, word: &str) -> bool {
    if !matches!(tail, '(' | ',')
        && !prefix.ends_with("||")
        && !["LIKE", "ILIKE", "BETWEEN", "AND"]
            .iter()
            .any(|keyword| word.eq_ignore_ascii_case(keyword))
    {
        return false;
    }
    let Ok(mut tokens) = Tokenizer::new(&PostgreSqlDialect {}, prefix).tokenize() else {
        return false;
    };
    tokens.retain(|token| !matches!(token, Token::Whitespace(_)));
    let Some((last, before)) = tokens.split_last() else {
        return false;
    };
    match last {
        Token::StringConcat => true,
        Token::Word(word) => match word.keyword {
            Keyword::LIKE | Keyword::ILIKE | Keyword::BETWEEN => has_left_operand(before),
            Keyword::AND => finishes_between_lower_bound(before),
            _ => false,
        },
        Token::LParen => group_values(before),
        Token::Comma => open_group(before).is_some_and(|open| group_values(&before[..open])),
        _ => false,
    }
}

fn has_left_operand(mut tokens: &[Token]) -> bool {
    if matches!(tokens.last(), Some(Token::Word(word)) if word.keyword == Keyword::NOT) {
        tokens = &tokens[..tokens.len() - 1];
    }
    match tokens.last() {
        Some(Token::Word(word)) => !matches!(
            word.keyword,
            Keyword::SELECT
                | Keyword::FROM
                | Keyword::JOIN
                | Keyword::WHERE
                | Keyword::BY
                | Keyword::INTO
                | Keyword::UPDATE
                | Keyword::AS
                | Keyword::AND
                | Keyword::OR
        ),
        Some(
            Token::Period | Token::DoubleColon | Token::LParen | Token::Comma | Token::SemiColon,
        )
        | None => false,
        _ => true,
    }
}

fn finishes_between_lower_bound(tokens: &[Token]) -> bool {
    let mut depth = 0;
    for (index, token) in tokens.iter().enumerate().rev() {
        match token {
            Token::RParen | Token::RBracket => depth += 1,
            Token::LParen | Token::LBracket if depth > 0 => depth -= 1,
            Token::LParen | Token::LBracket | Token::Comma | Token::SemiColon if depth == 0 => {
                return false;
            }
            Token::Word(word) if depth == 0 => match word.keyword {
                Keyword::BETWEEN => return has_left_operand(&tokens[..index]),
                Keyword::AND
                | Keyword::OR
                | Keyword::WHERE
                | Keyword::HAVING
                | Keyword::ON
                | Keyword::WHEN
                | Keyword::THEN
                | Keyword::ELSE => return false,
                _ => {}
            },
            _ => {}
        }
    }
    false
}

fn group_values(mut before: &[Token]) -> bool {
    loop {
        if matches!(before.last(), Some(Token::LParen)) {
            before = &before[..before.len() - 1];
        } else if let [earlier @ .., Token::RParen, Token::Comma] = before {
            let Some(open) = open_group(earlier) else {
                return false;
            };
            before = &earlier[..open];
        } else {
            break;
        }
    }
    let comparison = matches!(
        before.last(),
        Some(Token::Eq | Token::Neq | Token::Lt | Token::LtEq | Token::Gt | Token::GtEq)
    );
    (comparison && tuple_operand(&before[..before.len() - 1]))
        || matches!(before.last(), Some(Token::Word(word)) if word.keyword == Keyword::VALUES)
}

fn tuple_operand(tokens: &[Token]) -> bool {
    let Some((Token::RParen, inner)) = tokens.split_last() else {
        return false;
    };
    let Some(open) = open_group(inner) else {
        return false;
    };
    if matches!(inner.get(open.wrapping_sub(1)), Some(Token::Word(word))
        if !matches!(word.keyword, Keyword::WHERE | Keyword::HAVING | Keyword::ON
            | Keyword::AND | Keyword::OR | Keyword::NOT | Keyword::SELECT
            | Keyword::WHEN | Keyword::THEN | Keyword::ELSE | Keyword::ROW))
    {
        return false;
    }
    let mut depth = 0;
    for token in &inner[open + 1..] {
        match token {
            Token::LParen | Token::LBracket => depth += 1,
            Token::RParen | Token::RBracket => depth -= 1,
            Token::Comma if depth == 0 => return true,
            _ => {}
        }
    }
    false
}

fn open_group(tokens: &[Token]) -> Option<usize> {
    let mut depth = 0;
    for (index, token) in tokens.iter().enumerate().rev() {
        match token {
            Token::RParen => depth += 1,
            Token::LParen if depth == 0 => return Some(index),
            Token::LParen => depth -= 1,
            _ => {}
        }
    }
    None
}
