use super::super::Locations;
use crate::codebase::postgres::statements::{SqlFactSite as Site, SqlVariantLocations};
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Span, Token, TokenWithSpan};

pub(super) fn collect(index: &Locations, tokens: &[TokenWithSpan], out: &mut SqlVariantLocations) {
    for (i, (metadata, span)) in index.locking.iter().enumerate() {
        let (line, column) = lock_position(tokens, *span)
            .unwrap_or((span.start.line as usize, span.start.column as usize));
        out.insert(Site::Lock(i), line, column);
        out.locking.push(metadata.clone());
    }
    let words: Vec<_> = tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect();
    for (i, pair) in words
        .windows(2)
        .filter(|pair| {
            matches!(&pair[0].token, Token::Word(word) if word.keyword == Keyword::ON)
                && matches!(&pair[1].token, Token::Word(word) if word.keyword == Keyword::CONFLICT)
        })
        .enumerate()
    {
        let at = pair[0].span.start;
        out.insert(Site::Conflict(i), at.line as usize, at.column as usize);
    }
}
fn lock_position(tokens: &[TokenWithSpan], span: Span) -> Option<(usize, usize)> {
    let mut depth = 0usize;
    let mut owner_depth = None;
    for token in tokens {
        if token.span.start >= span.start && owner_depth.is_none() {
            owner_depth = Some(depth);
        }
        if token.span.start >= span.end
            && owner_depth == Some(depth)
            && matches!(&token.token, Token::Word(word) if word.keyword == Keyword::FOR)
        {
            return Some((
                token.span.start.line as usize,
                token.span.start.column as usize,
            ));
        }
        match token.token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}
