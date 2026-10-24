//! sqlparser has SHARE locks but lacks PostgreSQL's KEY SHARE spelling.
use sqlparser::tokenizer::{Token, TokenWithSpan};

fn key_share_keys(tokens: &[&Token]) -> Vec<usize> {
    let significant: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token, Token::Whitespace(_)))
        .collect();
    significant
        .windows(3)
        .filter_map(|triple| {
            (word(triple[0].1, "FOR") && word(triple[1].1, "KEY") && word(triple[2].1, "SHARE"))
                .then_some(triple[1].0)
        })
        .collect()
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

pub(super) fn compatible_tokens(tokens: &[&Token]) -> Vec<Token> {
    let skip = key_share_keys(tokens);
    tokens
        .iter()
        .enumerate()
        .filter(|(at, _)| !skip.contains(at))
        .map(|(_, token)| (**token).clone())
        .collect()
}

pub(super) fn rewrite_located(tokens: &mut Vec<TokenWithSpan>) {
    let borrowed: Vec<_> = tokens.iter().map(|token| &token.token).collect();
    for at in key_share_keys(&borrowed).into_iter().rev() {
        tokens.remove(at);
    }
}
