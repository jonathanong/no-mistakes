use super::super::LocatedStatement;
use super::queries;
use sqlparser::ast::{DiscardObject, Statement};
use sqlparser::tokenizer::{Token, TokenWithSpan};

pub(super) fn guarded_action(
    tokens: &[Token],
    original: Option<&[TokenWithSpan]>,
) -> Option<LocatedStatement> {
    let start = queries::guarded_action_start(tokens)?;
    let suffix = original?.get(start..)?;
    let mut marker = LocatedStatement::functions(
        Statement::Discard {
            object_type: DiscardObject::ALL,
        },
        false,
    );
    marker.recovery_failure = unrecoverable_chunk(suffix, 0);
    marker.recovery_failure.as_mut()?.source_order.clear();
    Some(marker)
}

pub(super) fn unrecoverable_chunk(
    tokens: &[TokenWithSpan],
    ordinal: usize,
) -> Option<super::super::super::super::types::SqlUnanalyzableStatement> {
    let significant = tokens
        .iter()
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect::<Vec<_>>();
    // A procedural opener can share a semicolon chunk with the first SQL action.
    // Anchor a failure to that action, retaining the opener alone for its own marker.
    let start = usize::from(significant.len() > 1 && significant.first().is_some_and(|token| {
        matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && ["BEGIN", "ELSE", "LOOP"].iter().any(|prefix| word.value.eq_ignore_ascii_case(prefix)))
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
