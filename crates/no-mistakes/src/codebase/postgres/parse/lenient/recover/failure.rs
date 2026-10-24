use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn unrecoverable_chunk(
    tokens: &[TokenWithSpan],
    ordinal: usize,
) -> Option<super::super::super::super::types::SqlUnanalyzableStatement> {
    let significant = tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect::<Vec<_>>();
    // Peeled DO bodies can place procedural BEGIN before the first SQL action
    // in the same semicolon chunk. Anchor the failure to that action.
    let start = usize::from(significant.first().is_some_and(|token| {
        matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case("BEGIN"))
    }));
    let statement = significant.get(start..)?;
    let line = statement.first()?.span.start.line as usize;
    let head = statement
        .iter()
        .take(8)
        .map(|token| token.token.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    Some(
        super::super::super::super::types::SqlUnanalyzableStatement {
            line,
            head,
            source_order: vec![ordinal],
        },
    )
}
