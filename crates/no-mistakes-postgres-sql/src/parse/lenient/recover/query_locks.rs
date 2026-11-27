//! Adapt PostgreSQL row lock spellings while retaining their row/query structure.
use sqlparser::tokenizer::{Token, TokenWithSpan};

fn lock_modifiers(tokens: &[&Token]) -> Vec<usize> {
    let significant: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token, Token::Whitespace(_)))
        .collect();
    let mut skip = Vec::new();
    for part in significant.windows(4) {
        if word(part[0].1, "FOR")
            && word(part[1].1, "NO")
            && word(part[2].1, "KEY")
            && word(part[3].1, "UPDATE")
        {
            skip.extend([part[1].0, part[2].0]);
        }
    }
    skip.extend(
        significant
            .windows(3)
            .filter_map(|triple| {
                (word(triple[0].1, "FOR") && word(triple[1].1, "KEY") && word(triple[2].1, "SHARE"))
                    .then_some(triple[1].0)
            })
            .collect::<Vec<_>>(),
    );
    skip.sort_unstable();
    skip.dedup();
    skip
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

pub(super) fn compatible_tokens(tokens: &[&Token]) -> Vec<Token> {
    let skip = lock_modifiers(tokens);
    tokens
        .iter()
        .enumerate()
        .filter(|(at, _)| !skip.contains(at))
        .map(|(_, token)| (**token).clone())
        .collect()
}

pub(super) fn rewrite_located(tokens: &mut Vec<TokenWithSpan>) {
    let borrowed: Vec<_> = tokens.iter().map(|token| &token.token).collect();
    for at in lock_modifiers(&borrowed).into_iter().rev() {
        tokens.remove(at);
    }
}
