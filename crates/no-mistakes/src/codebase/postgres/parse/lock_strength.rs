use sqlparser::tokenizer::{Token, TokenWithSpan};

/// sqlparser rejects `FOR NO KEY UPDATE` and `FOR KEY SHARE`. Lock ordering
/// depends on which rows are locked and in what order, not on the strength, so
/// rewrite them to `FOR UPDATE` / `FOR SHARE` by dropping the `NO KEY` / `KEY`
/// words. Surviving tokens keep their original spans.
pub(super) fn normalize(tokens: &mut Vec<TokenWithSpan>) {
    let mut drop = vec![false; tokens.len()];
    let mut changed = false;
    for start in 0..tokens.len() {
        if is_word(&tokens[start].token, "for") {
            changed |= mark_prefix(tokens, start, &mut drop);
        }
    }
    if changed {
        let mut index = 0;
        tokens.retain(|_| {
            index += 1;
            !drop[index - 1]
        });
    }
}

/// Mark the `NO KEY` / `KEY` words after the `FOR` at `start`, plus the
/// whitespace following each, for removal.
fn mark_prefix(tokens: &[TokenWithSpan], start: usize, drop: &mut [bool]) -> bool {
    let mut words: Vec<(usize, String)> = Vec::new();
    let mut at = start + 1;
    while words.len() < 3 {
        while matches!(tokens.get(at), Some(t) if matches!(t.token, Token::Whitespace(_))) {
            at += 1;
        }
        match tokens.get(at).map(|t| &t.token) {
            Some(Token::Word(word)) if word.quote_style.is_none() => {
                words.push((at, word.value.to_ascii_lowercase()));
                at += 1;
            }
            _ => break,
        }
    }
    let names: Vec<&str> = words.iter().map(|(_, name)| name.as_str()).collect();
    let removed = match names.as_slice() {
        ["no", "key", "update"] => &words[..2],
        ["key", "share", ..] => &words[..1],
        _ => return false,
    };
    for (index, _) in removed {
        drop[*index] = true;
        drop[index + 1] = true;
    }
    true
}

fn is_word(token: &Token, keyword: &str) -> bool {
    matches!(token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(keyword))
}

#[cfg(test)]
mod tests;
