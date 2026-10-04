use super::{normalize_copy_data, normalize_table_queries, PostgresParseError};
use sqlparser::ast::Statement;
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::{TokenWithSpan, Tokenizer};
use std::borrow::Cow;
use std::cell::{Cell, OnceCell};

/// One request's aligned SQL text and located tokens. Parser rewrites use a clone;
/// row-bound identity and sweeps borrow the original located stream.
pub(crate) struct PreparedSql<'a> {
    normalized: Cow<'a, str>,
    source_positions_preserved: Cell<bool>,
    tokens: OnceCell<Result<Vec<TokenWithSpan>, PostgresParseError>>,
}

impl<'a> PreparedSql<'a> {
    pub(crate) fn new(sql: &'a str) -> Self {
        Self {
            normalized: normalize_copy_data(sql),
            tokens: OnceCell::new(),
            source_positions_preserved: Cell::new(true),
        }
    }

    pub(crate) fn normalized(&self) -> &str {
        &self.normalized
    }

    pub(crate) fn tokens(&self) -> &[TokenWithSpan] {
        self.located().map_or(&[], Vec::as_slice)
    }

    pub(crate) fn tokens_preserve_source_positions(&self) -> bool {
        let _ = self.located();
        self.source_positions_preserved.get()
    }

    pub(crate) fn parse(&self) -> Result<Vec<Statement>, PostgresParseError> {
        let located = self.located().map_err(Clone::clone)?;
        let mut tokens = super::radix_numbers::repair(located).unwrap_or_else(|| located.clone());
        normalize_table_queries(&mut tokens);
        let fetch = super::fetch_expression::prepare(&mut tokens);
        let mut statements = Parser::new(&PostgreSqlDialect {})
            .with_tokens_with_locations(tokens)
            .parse_statements()
            .map_err(PostgresParseError::from)?;
        super::fetch_expression::restore(&mut statements, &fetch);
        Ok(statements)
    }

    fn located(&self) -> Result<&Vec<TokenWithSpan>, &PostgresParseError> {
        self.tokens
            .get_or_init(|| {
                let separated = super::distinct_group::separate_distinct_grouping(&self.normalized);
                self.source_positions_preserved
                    .set(matches!(separated, Cow::Borrowed(_)));
                Tokenizer::new(&PostgreSqlDialect {}, &separated)
                    .tokenize_with_location()
                    .map_err(|error| PostgresParseError {
                        message: error.to_string(),
                    })
            })
            .as_ref()
    }
}
