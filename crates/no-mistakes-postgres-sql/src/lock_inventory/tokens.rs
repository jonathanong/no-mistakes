use super::{advisory_name, record, InventoryLockFact};
use sqlparser::ast::{Lock, Query, Spanned};
use sqlparser::tokenizer::{Location, Span, Token, TokenWithSpan};
use std::cell::OnceCell;

mod advisory;
mod fingerprint;
mod restoration;
mod row;
pub(crate) use row::RowLockMode;

pub(crate) struct Segments<'a> {
    segments: Vec<Segment<'a>>,
}

struct Segment<'a> {
    tokens: Vec<Lexeme<'a>>,
    fingerprint: OnceCell<String>,
}

struct Lexeme<'a> {
    token: &'a Token,
    span: Span,
    depth: usize,
}

impl<'a> Segments<'a> {
    pub(crate) fn new(tokens: &'a [TokenWithSpan]) -> Self {
        Self::build(tokens, None)
    }

    pub(super) fn recovered(
        tokens: &'a [TokenWithSpan],
        source_at: &dyn Fn(Span) -> Option<&'a str>,
    ) -> Self {
        Self::build(tokens, Some(source_at))
    }

    fn build(
        tokens: &'a [TokenWithSpan],
        source_at: Option<&dyn Fn(Span) -> Option<&'a str>>,
    ) -> Self {
        let segments = tokens
            .split(|token| token.token == Token::SemiColon)
            .filter_map(|tokens| {
                let advisory = tokens.iter().any(advisory::advisory_token);
                (advisory
                    || tokens
                        .iter()
                        .any(|token| word(&token.token, "for") || word(&token.token, "lock")))
                .then(|| Segment::new(tokens, source_at))
            })
            .collect();
        Self { segments }
    }

    pub(super) fn has_candidates_at(&self, at: Location) -> bool {
        if at.line == 0 {
            return !self.segments.is_empty();
        }
        self.at(at).is_some()
    }

    fn at(&self, at: Location) -> Option<&Segment<'a>> {
        let index = self
            .segments
            .partition_point(|segment| segment.start() <= at);
        let segment = self.segments.get(index.checked_sub(1)?)?;
        (at < segment.end()).then_some(segment)
    }

    pub(crate) fn row_lock_modes(&self, query: &Query) -> Option<Vec<RowLockMode>> {
        row::modes(self.at(query.span().start)?, query)
    }

    pub(super) fn row_locks(&self, query: &Query, out: &mut Vec<InventoryLockFact>) -> bool {
        self.at(query.span().start)
            .and_then(|segment| row::collect(segment, query, out))
            .is_some()
    }

    pub(super) fn table_lock(&self, lock: &Lock, out: &mut Vec<InventoryLockFact>) -> Option<()> {
        let at = lock.tables[0].name.span().start;
        let segment = self.at(at)?;
        let index = segment.index_at(at);
        let head = segment.tokens[..index]
            .iter()
            .rev()
            .find(|lexeme| !lexeme.word("only") && !lexeme.word("table"))
            .filter(|lexeme| lexeme.word("lock"))?;
        let wait = if lock.nowait {
            let end = lock.tables.last().unwrap().name.span().end;
            Some(
                segment.tokens[segment.index_at(end)..]
                    .iter()
                    .find(|lexeme| lexeme.word("nowait"))?,
            )
        } else {
            None
        };
        segment.record("lock-table", head, out);
        if let Some(wait) = wait {
            segment.record("nowait", wait, out);
        }
        Some(())
    }
}

impl<'a> Segment<'a> {
    fn new(
        tokens: &'a [TokenWithSpan],
        source_at: Option<&dyn Fn(Span) -> Option<&'a str>>,
    ) -> Self {
        let mut tokens: Vec<_> = tokens
            .iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_) | Token::EOF))
            .map(|source| Lexeme {
                token: &source.token,
                span: source.span,
                depth: 0,
            })
            .collect();
        if let Some(source_at) = source_at {
            tokens = restoration::restore(tokens, source_at);
        }
        let mut depth = 0usize;
        for token in &mut tokens {
            if matches!(token.token(), Token::RParen | Token::RBracket) {
                depth = depth.saturating_sub(1);
            }
            token.depth = depth;
            if matches!(token.token(), Token::LParen | Token::LBracket) {
                depth += 1;
            }
        }
        Self {
            tokens,
            fingerprint: OnceCell::new(),
        }
    }

    fn start(&self) -> Location {
        self.tokens[0].at()
    }

    fn end(&self) -> Location {
        self.tokens.last().unwrap().span.end
    }

    fn index_at(&self, at: Location) -> usize {
        self.tokens.partition_point(|token| token.at() < at)
    }

    fn record(&self, category: &str, lexeme: &Lexeme<'_>, out: &mut Vec<InventoryLockFact>) {
        let normalized = self
            .fingerprint
            .get_or_init(|| fingerprint::normalize(&self.tokens));
        record(
            category,
            normalized,
            lexeme.at(),
            Span::new(self.start(), self.end()),
            out,
        );
    }
}

impl Lexeme<'_> {
    fn at(&self) -> Location {
        self.span.start
    }

    fn token(&self) -> &Token {
        self.token
    }

    fn word(&self, expected: &str) -> bool {
        word(self.token(), expected)
    }
}

fn word(token: &Token, expected: &str) -> bool {
    matches!(token, Token::Word(word)
        if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
