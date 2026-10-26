use super::word;
use sqlparser::tokenizer::Token;

pub(super) fn unlabeled<'a>(code: &'a [&'a Token]) -> &'a [&'a Token] {
    if let [Token::ShiftLeft, Token::Word(_), Token::ShiftRight, rest @ ..] = code {
        // PostgreSQL permits opening labels on blocks and loops, not IF or SQL actions.
        if rest.first().is_some_and(|token| {
            ["BEGIN", "DECLARE", "FOR", "WHILE", "FOREACH", "LOOP"]
                .iter()
                .any(|kind| word(token, kind))
        }) {
            return rest;
        }
    }
    code
}
