use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Whitespace};

/// sqlparser 0.63 parses `TABLE name` but does not accept PostgreSQL's ONLY
/// modifier. Keep its source location while hiding the modifier from the AST.
pub(super) fn normalize(tokens: &mut [TokenWithSpan]) {
    let words: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| !matches!(token.token, Token::Whitespace(_)))
        .map(|(index, _)| index)
        .collect();
    for (position, &index) in words.iter().enumerate() {
        let Token::Word(table) = &tokens[index].token else {
            continue;
        };
        if table.keyword != Keyword::TABLE
            || !query_context(
                position
                    .checked_sub(1)
                    .map(|before| &tokens[words[before]].token),
            )
        {
            continue;
        }
        let Some(&only_index) = words.get(position + 1) else {
            continue;
        };
        if matches!(&tokens[only_index].token, Token::Word(word) if word.quote_style.is_none() && word.keyword == Keyword::ONLY)
        {
            tokens[only_index].token = Token::Whitespace(Whitespace::Space);
        }
    }
}

fn query_context(previous: Option<&Token>) -> bool {
    match previous {
        None | Some(Token::SemiColon | Token::LParen | Token::RParen) => true,
        Some(Token::Word(word)) => matches!(
            word.value.to_ascii_uppercase().as_str(),
            "ALL" | "DISTINCT" | "UNION" | "INTERSECT" | "EXCEPT" | "AS"
        ),
        _ => false,
    }
}
