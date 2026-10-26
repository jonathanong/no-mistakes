//! Render recovery heads from source lexemes before compatibility rewrites.
use super::distinct_group::HeadSourceMap;
use super::LocatedStatement;
use sqlparser::tokenizer::{Location, Span, Token, TokenWithSpan};
mod lines;
use lines::SourceLines;

pub(super) struct OriginalHeads(Vec<HeadToken>);

#[derive(Clone, Copy)]
struct HeadToken {
    span: Span,
    whitespace: bool,
    terminal: bool,
}

impl OriginalHeads {
    pub(super) fn capture(tokens: &[TokenWithSpan]) -> Self {
        Self(
            tokens
                .iter()
                .map(|token| HeadToken {
                    span: token.span,
                    whitespace: matches!(token.token, Token::Whitespace(_)),
                    terminal: matches!(token.token, Token::SemiColon | Token::EOF),
                })
                .collect(),
        )
    }

    pub(super) fn restore(
        &self,
        sql: &str,
        normalized: &str,
        tokenized: &str,
        map: &HeadSourceMap,
        statements: &mut [LocatedStatement],
    ) {
        if !statements
            .iter()
            .any(|statement| statement.recovery_failure.is_some())
        {
            return;
        }
        restore(sql, normalized, tokenized, map, &self.0, statements);
    }
}

fn restore(
    sql: &str,
    normalized: &str,
    tokenized: &str,
    map: &HeadSourceMap,
    tokens: &[HeadToken],
    statements: &mut [LocatedStatement],
) {
    let token_lines = SourceLines::new(tokenized);
    let normalized_lines = SourceLines::new(normalized);
    let source_lines = SourceLines::new(sql);
    for statement in statements {
        let Some(failure) = &mut statement.recovery_failure else {
            continue;
        };
        let location = Location::new(failure.line as u64, failure.column as u64);
        let at = tokens.partition_point(|token| token.span.start < location);
        if tokens
            .get(at)
            .is_none_or(|token| token.span.start != location)
        {
            continue;
        }
        let restored = (|| {
            let token = &tokens[at];
            let start = token_lines.offset(token.span.start)?;
            let end = token_lines.offset(token.span.end)?;
            let (source, _) = raw_span(start, end, map, &normalized_lines, &source_lines)?;
            let original = source_lines.location(source)?;
            let mut head = String::new();
            let mut count = 0;
            let mut at = at;
            while count < 8 {
                let Some(token) = tokens.get(at) else { break };
                if token.terminal {
                    break;
                }
                at += 1;
                if token.whitespace {
                    continue;
                }
                let start = token_lines.offset(token.span.start)?;
                let token_end = token_lines.offset(token.span.end)?;
                let Some((start, end)) =
                    raw_span(start, token_end, map, &normalized_lines, &source_lines)
                else {
                    continue;
                };
                let value = sql.get(start..end)?;
                if !head.is_empty() {
                    head.push(' ');
                }
                head.push_str(value);
                count += 1;
                // The lexer can split a radix literal into adjacent `0` and `x1F`,
                // `b101`, or `o77` tokens. It remains one source lexeme in a head.
                if value == "0" {
                    if let Some(next) = tokens.get(at) {
                        if next.span.start == token.span.end {
                            let tail_end = token_lines.offset(next.span.end)?;
                            let (_, tail_end) = raw_span(
                                token_end,
                                tail_end,
                                map,
                                &normalized_lines,
                                &source_lines,
                            )?;
                            let tail = sql.get(end..tail_end)?;
                            if tail
                                .chars()
                                .next()
                                .is_some_and(|c| matches!(c, 'x' | 'X' | 'b' | 'B' | 'o' | 'O'))
                            {
                                head.push_str(tail);
                                at += 1;
                            }
                        }
                    }
                }
            }
            Some((original, head))
        })();
        if let Some((original, head)) = restored.filter(|(_, head)| !head.is_empty()) {
            // Commit text and location together; mapping drift preserves both.
            failure.line = original.line as usize;
            failure.column = original.column as usize;
            failure.head = head;
        }
    }
}

fn raw_span(
    start: usize,
    end: usize,
    map: &HeadSourceMap,
    normalized: &SourceLines<'_>,
    source: &SourceLines<'_>,
) -> Option<(usize, usize)> {
    let (start, end) = map.source_span(start, end)?;
    let start = source.offset(normalized.location(start)?)?;
    let end = source.offset(normalized.location(end)?)?;
    Some((start, end))
}

#[cfg(test)]
mod tests;
