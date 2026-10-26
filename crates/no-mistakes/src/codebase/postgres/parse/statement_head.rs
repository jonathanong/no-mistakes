use sqlparser::tokenizer::{Location, Token, TokenWithSpan};

/// The diagnostic head is bounded to eight original tokens after a procedural opener.
pub(super) fn head(tokens: &[TokenWithSpan]) -> Option<(Location, String)> {
    let mut significant = tokens
        .iter()
        .take_while(|token| !matches!(token.token, Token::SemiColon | Token::EOF))
        .filter(|token| !matches!(token.token, Token::Whitespace(_)));
    // A procedural opener can share a semicolon chunk with the first SQL action.
    // Anchor a failure to that action, retaining the opener alone for its own marker.
    let location = loop {
        let mut lookahead = significant.clone();
        let first = lookahead.next()?;
        let skip = if matches!(&first.token, Token::ShiftLeft)
            && lookahead
                .next()
                .is_some_and(|token| matches!(&token.token, Token::Word(_)))
            && lookahead
                .next()
                .is_some_and(|token| token.token == Token::ShiftRight)
            && lookahead.next().is_some()
        {
            3
        } else if matches!(&first.token, Token::Word(word)
            if word.quote_style.is_none()
                && ["BEGIN", "ELSE", "LOOP"].iter().any(|prefix| word.value.eq_ignore_ascii_case(prefix)))
            && lookahead.next().is_some()
        {
            1
        } else {
            break first.span.start;
        };
        significant.nth(skip - 1);
    };
    let head = significant
        .take(8)
        .map(|token| token.token.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    Some((location, head))
}
