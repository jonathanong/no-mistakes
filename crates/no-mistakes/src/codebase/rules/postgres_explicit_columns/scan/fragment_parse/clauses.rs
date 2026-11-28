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
