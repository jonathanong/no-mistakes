use super::super::tokens::Tokens;
use super::{keyword_pair_words, Word};
use sqlparser::ast::{Insert, Spanned};

pub(in crate::codebase::postgres::statements) struct InsertSources<'a> {
    tokens: Option<&'a Tokens<'a>>,
    sql: &'a str,
    pairs: std::cell::OnceCell<Vec<Word>>,
}

impl<'a> InsertSources<'a> {
    pub(in crate::codebase::postgres::statements) fn new(sql: &'a str) -> Self {
        Self {
            tokens: None,
            sql,
            pairs: std::cell::OnceCell::new(),
        }
    }

    pub(in crate::codebase::postgres::statements) fn with_tokens(
        sql: &'a str,
        tokens: &'a Tokens<'a>,
    ) -> Self {
        Self {
            tokens: Some(tokens),
            ..Self::new(sql)
        }
    }

    pub(in crate::codebase::postgres::statements) fn for_insert(
        &self,
        insert: &Insert,
        n: usize,
    ) -> &str {
        let mut span = insert.span();
        span.start = insert.insert_token.0.span.start;
        if let Some(source) = &insert.source {
            let body = source.body.span().start;
            if body > span.start {
                // OVERRIDING belongs to this head, never a nested INSERT source.
                span.end = body;
            }
        }
        match self.tokens {
            Some(tokens) => tokens.source_at(span).unwrap_or_default(),
            None => self.source(n),
        }
    }

    pub(in crate::codebase::postgres::statements) fn line(
        &self,
        insert: &Insert,
        n: usize,
    ) -> usize {
        if self.tokens.is_some() {
            insert.insert_token.0.span.start.line.max(1) as usize
        } else {
            self.pairs()
                .get(n.saturating_sub(1))
                .map_or(1, |word| word.line)
        }
    }

    fn pairs(&self) -> &[Word] {
        self.pairs
            .get_or_init(|| keyword_pair_words(self.sql, "insert", "into"))
    }

    pub(in crate::codebase::postgres::statements) fn source(&self, n: usize) -> &str {
        let start = self
            .pairs()
            .get(n.saturating_sub(1))
            .map_or(0, |word| word.start);
        let end = self
            .pairs()
            .get(n)
            .map_or(self.sql.len(), |word| word.start);
        self.sql.get(start..end).unwrap_or_default()
    }
}
