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

/// Place a peeled `DO` body on the opening dollar quote's line and column.
pub(super) fn align_do_body(body: &str, original: Option<&[TokenWithSpan]>) -> String {
    let Some((line, column, tag)) = original.and_then(dollar_quote) else {
        return body.to_string();
    };
    let opener = 2 + tag.chars().count();
    format!(
        "{}{}{body}",
        "\n".repeat(line.saturating_sub(1)),
        " ".repeat(column.saturating_add(opener).saturating_sub(1))
    )
}

/// Flatten decoded `chr()` text onto the chunk's first source line.
pub(super) fn align_chr_sql(sql: &str, original: Option<&[TokenWithSpan]>) -> String {
    let flat = sql.replace(['\n', '\r'], " ");
    format!(
        "{}{flat}",
        "\n".repeat(chunk_line(original).saturating_sub(1))
    )
}

fn dollar_quote(tokens: &[TokenWithSpan]) -> Option<(usize, usize, &str)> {
    tokens.iter().find_map(|token| match &token.token {
        Token::DollarQuotedString(quoted) => Some((
            token.span.start.line as usize,
            token.span.start.column as usize,
            quoted.tag.as_deref().unwrap_or(""),
        )),
        _ => None,
    })
}

fn chunk_line(original: Option<&[TokenWithSpan]>) -> usize {
    original
        .and_then(|tokens| {
            tokens
                .iter()
                .find(|token| !matches!(token.token, Token::Whitespace(_)))
        })
        .map(|token| token.span.start.line as usize)
        .unwrap_or(1)
}
