use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan};

// Rewrites delete tokens or insert/replace generated-column STORED.
pub(super) fn align(tokens: &[Token], original: &[TokenWithSpan]) -> Vec<TokenWithSpan> {
    let mut at = 0;
    tokens
        .iter()
        .map(|token| {
            let stored = matches!(token, Token::Word(word) if word.keyword == Keyword::STORED);
            let virtual_ = original.get(at).is_some_and(|token| {
            matches!(&token.token, Token::Word(word) if word.keyword == Keyword::VIRTUAL)
        });
            let inserted =
                stored && !virtual_ && original.get(at).is_none_or(|source| source.token != *token);
            if !(inserted || stored && virtual_) {
                while at < original.len() && original[at].token != *token {
                    at += 1;
                }
            }
            // parse_chunks supplies only nonempty source chunks.
            let source = original.get(at).unwrap_or(&original[original.len() - 1]);
            let value = TokenWithSpan::new(token.clone(), source.span);
            if !inserted {
                at += 1;
            }
            value
        })
        .collect()
}
