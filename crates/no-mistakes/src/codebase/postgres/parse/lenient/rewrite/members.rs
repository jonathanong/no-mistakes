use super::column::is_column_word;
use super::{keyword_of, next_non_ws};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, Word};

pub(super) fn action_list_matches_foreign_key(
    tokens: &[Token],
    open: usize,
    end: usize,
    on_at: usize,
) -> bool {
    let Some(names) = action_names(tokens, open, end) else {
        return false;
    };
    let Some(local) = referencing_columns(tokens, on_at) else {
        return false;
    };
    names
        .iter()
        .all(|name| local.iter().any(|column| same_column(name, column)))
}

fn action_names(tokens: &[Token], open: usize, end: usize) -> Option<Vec<Word>> {
    let mut names = Vec::new();
    let mut expect_ident = true;
    for token in &tokens[open + 1..end - 1] {
        if matches!(token, Token::Whitespace(_)) {
            continue;
        }
        if expect_ident {
            let Token::Word(word) = token else {
                return None;
            };
            if !is_column_word(word) {
                return None;
            }
            names.push(word.clone());
            expect_ident = false;
        } else if matches!(token, Token::Comma) {
            expect_ident = true;
        } else {
            return None;
        }
    }
    (!names.is_empty() && !expect_ident).then_some(names)
}

fn referencing_columns(tokens: &[Token], on_at: usize) -> Option<Vec<Word>> {
    let references = keyword_before(tokens, on_at, Keyword::REFERENCES)?;
    if let Some(columns) = foreign_key_columns(tokens, references) {
        return Some(columns);
    }
    column_constraint_name(tokens, references).map(|name| vec![name])
}

fn foreign_key_columns(tokens: &[Token], references_at: usize) -> Option<Vec<Word>> {
    let foreign = keyword_before(tokens, references_at, Keyword::FOREIGN)?;
    let key = next_non_ws(tokens, foreign + 1)?;
    if keyword_of(tokens.get(key)?) != Some(Keyword::KEY) {
        return None;
    }
    let open = next_non_ws(tokens, key + 1)?;
    if !matches!(tokens.get(open)?, Token::LParen) {
        return None;
    }
    let end = super::super::skip_balanced_parens(tokens, open)?;
    action_names(tokens, open, end)
}

fn column_constraint_name(tokens: &[Token], references_at: usize) -> Option<Word> {
    let start = element_start(tokens, references_at);
    tokens[start..references_at].iter().find_map(|token| {
        let Token::Word(word) = token else {
            return None;
        };
        is_column_word(word).then(|| word.clone())
    })
}

fn same_column(action: &Word, column: &Word) -> bool {
    folded_name(action) == folded_name(column)
}

fn folded_name(word: &Word) -> String {
    if word.quote_style.is_some() {
        word.value.clone()
    } else {
        word.value.to_ascii_lowercase()
    }
}

fn keyword_before(tokens: &[Token], before: usize, keyword: Keyword) -> Option<usize> {
    let mut index = before;
    let mut depth = 0i32;
    while index > 0 {
        index -= 1;
        if depth == 0 && keyword_of(&tokens[index]) == Some(keyword) {
            return Some(index);
        }
        if at_element_boundary(&tokens[index], &mut depth) {
            return None;
        }
    }
    None
}

fn element_start(tokens: &[Token], before: usize) -> usize {
    let mut index = before;
    let mut depth = 0i32;
    while index > 0 {
        index -= 1;
        if at_element_boundary(&tokens[index], &mut depth) {
            return index + 1;
        }
    }
    0
}

fn at_element_boundary(token: &Token, depth: &mut i32) -> bool {
    match token {
        Token::RParen => *depth += 1,
        Token::LParen => {
            *depth -= 1;
            if *depth < 0 {
                return true;
            }
        }
        Token::SemiColon | Token::Comma if *depth == 0 => return true,
        _ => {}
    }
    false
}
