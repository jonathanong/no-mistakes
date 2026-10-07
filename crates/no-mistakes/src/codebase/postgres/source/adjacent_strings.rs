//! PostgreSQL joins plain string constants only across whitespace with a newline.
use sqlparser::tokenizer::{Token, TokenWithSpan, Whitespace};

pub(super) fn prepare(tokens: &mut Vec<TokenWithSpan>) {
    let mut output: Vec<TokenWithSpan> = Vec::with_capacity(tokens.len());
    let mut input = std::mem::take(tokens).into_iter().peekable();
    while let Some(mut token) = input.next() {
        if let Token::SingleQuotedString(value) = &mut token.token {
            loop {
                let mut gap = Vec::new();
                // PostgreSQL's quote-continuation scanner accepts line comments,
                // but a block comment terminates the literal even with a newline.
                while input.peek().is_some_and(|next| {
                    matches!(
                        next.token,
                        Token::Whitespace(
                            Whitespace::Space
                                | Whitespace::Tab
                                | Whitespace::Newline
                                | Whitespace::SingleLineComment { .. }
                        )
                    )
                }) {
                    gap.push(input.next().unwrap());
                }
                let newline = gap
                    .iter()
                    .any(|part| part.span.end.line > part.span.start.line);
                if newline
                    && input
                        .peek()
                        .is_some_and(|next| matches!(next.token, Token::SingleQuotedString(_)))
                {
                    let next = input.next().unwrap();
                    if let Token::SingleQuotedString(part) = next.token {
                        value.push_str(&part);
                        // Remove all consumed gap tokens so token ends stay monotonic.
                        token.span.end = next.span.end;
                    }
                } else {
                    output.push(token);
                    output.extend(gap);
                    break;
                }
            }
        } else {
            output.push(token);
        }
    }
    *tokens = output;
}
