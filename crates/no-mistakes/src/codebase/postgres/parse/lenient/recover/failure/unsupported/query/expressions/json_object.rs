use super::{expression, word};
use sqlparser::tokenizer::Token;

pub(super) fn rewrite(tokens: &mut Vec<Token>, at: usize, end: usize) -> bool {
    let Some(with) = (at + 2..end.saturating_sub(1))
        .find(|pos| word(&tokens[*pos], "WITH") && word(&tokens[*pos + 1], "UNIQUE"))
    else {
        return false;
    };
    if !pairs(&tokens[at + 2..with]) {
        return false;
    }
    let after = with + 2 + usize::from(tokens.get(with + 2).is_some_and(|t| word(t, "KEYS")));
    tokens.drain(with..after);
    true
}

fn pairs(tokens: &[Token]) -> bool {
    let mut start = 0;
    let mut parens = 0;
    let mut brackets = 0;
    for (at, token) in tokens.iter().enumerate() {
        match token {
            Token::LParen => parens += 1,
            Token::RParen => parens -= 1,
            Token::LBracket => brackets += 1,
            Token::RBracket => brackets -= 1,
            Token::Comma if parens == 0 && brackets == 0 => {
                if !pair(&tokens[start..at], false) {
                    return false;
                }
                start = at + 1;
            }
            _ => {}
        }
        if parens < 0 || brackets < 0 {
            return false;
        }
    }
    parens == 0 && brackets == 0 && pair(&tokens[start..], true)
}

fn pair(tokens: &[Token], last: bool) -> bool {
    let value_end = if last
        && tokens.len() >= 3
        && (word(&tokens[tokens.len() - 3], "ABSENT") || word(&tokens[tokens.len() - 3], "NULL"))
        && word(&tokens[tokens.len() - 2], "ON")
        && word(&tokens[tokens.len() - 1], "NULL")
    {
        tokens.len() - 3
    } else {
        tokens.len()
    };
    let Some(colon) = (0..value_end).find(|&at| tokens[at] == Token::Colon) else {
        return false;
    };
    colon > 0
        && colon + 1 < value_end
        && expression(&tokens[..colon])
        && expression(&tokens[colon + 1..value_end])
}
