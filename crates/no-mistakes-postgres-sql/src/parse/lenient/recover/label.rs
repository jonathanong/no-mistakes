//! Strip a leading PL/pgSQL `<<label>>` so it cannot hide a transaction-control statement.
use sqlparser::tokenizer::Token;

pub(super) fn strip_label(prefix: &[Token]) -> &[Token] {
    let code: Vec<_> = prefix
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token, Token::Whitespace(_)))
        .take(3)
        .collect();
    match code.as_slice() {
        [(_, Token::ShiftLeft), (_, Token::Word(_)), (end, Token::ShiftRight)] => {
            &prefix[end + 1..]
        }
        _ => prefix,
    }
}
