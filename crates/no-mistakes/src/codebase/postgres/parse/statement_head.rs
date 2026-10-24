use sqlparser::tokenizer::{Location, Token, TokenWithSpan};

/// The diagnostic head is bounded to eight original tokens after a procedural opener.
pub(super) fn head(tokens: &[TokenWithSpan]) -> Option<(Location, String)> {
    let significant: Vec<_> = tokens
        .iter()
        .take_while(|token| !matches!(token.token, Token::SemiColon | Token::EOF))
        .filter(|token| !matches!(token.token, Token::Whitespace(_)))
        .collect();
    // A procedural opener can share a semicolon chunk with the first SQL action.
    // Anchor a failure to that action, retaining the opener alone for its own marker.
    let at = action_start(&significant);
    let location = significant.get(at)?.span.start;
    let head = significant[at..]
        .iter()
        .take(8)
        .map(|token| token.token.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    Some((location, head))
}

fn action_start(tokens: &[&TokenWithSpan]) -> usize {
    let mut at = 0;
    loop {
        let next = if matches!(
            tokens.get(at).map(|token| &token.token),
            Some(Token::ShiftLeft)
        ) && matches!(
            tokens.get(at + 1).map(|token| &token.token),
            Some(Token::Word(_))
        ) && matches!(
            tokens.get(at + 2).map(|token| &token.token),
            Some(Token::ShiftRight)
        ) {
            at + 3
        } else if ["BEGIN", "ELSE", "LOOP"]
            .iter()
            .any(|name| word(tokens, at, name))
        {
            at + 1
        } else if ["IF", "ELSIF", "WHEN", "CASE"]
            .iter()
            .any(|name| word(tokens, at, name))
        {
            boundary(tokens, at + 1, "THEN").map_or(at, |boundary| boundary + 1)
        } else {
            at
        };
        if next == at || next >= tokens.len() {
            return at;
        }
        at = next;
    }
}

fn boundary(tokens: &[&TokenWithSpan], start: usize, expected: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut cases = 0usize;
    for at in start..tokens.len() {
        match &tokens[at].token {
            Token::LParen => depth += 1,
            Token::RParen => depth = depth.saturating_sub(1),
            _ => {}
        }
        if word(tokens, at, "CASE") {
            cases += 1;
        } else if word(tokens, at, "END") {
            cases = cases.saturating_sub(1);
        } else if depth == 0 && cases == 0 && word(tokens, at, expected) {
            return Some(at);
        }
    }
    None
}

fn word(tokens: &[&TokenWithSpan], at: usize, name: &str) -> bool {
    matches!(tokens.get(at).map(|token| &token.token), Some(Token::Word(word))
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(name))
}
