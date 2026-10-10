//! Keep PostgreSQL comments out of adjacent custom operator tokens.
use super::distinct_group::{
    skip_comment_source as skip_comment, skip_opaque_source as skip_opaque,
};
use sqlparser::{
    dialect::{Dialect, PostgreSqlDialect},
    tokenizer::{Location, Token, TokenWithSpan, Tokenizer, TokenizerError, Whitespace},
};
use std::{borrow::Cow, collections::BTreeMap};

pub(super) struct Prepared<'a> {
    sql: Cow<'a, str>,
    inserted_spaces: Vec<Insertion>,
    columns_by_line: BTreeMap<u64, Vec<u64>>,
}

struct Insertion {
    offset: usize,
    prepared: Location,
}

impl<'a> Prepared<'a> {
    pub(super) fn new(sql: &'a str) -> Self {
        let inserted_spaces = separator_locations(sql);
        if inserted_spaces.is_empty() {
            return Self {
                sql: Cow::Borrowed(sql),
                inserted_spaces,
                columns_by_line: BTreeMap::new(),
            };
        }

        let mut projected = String::with_capacity(sql.len() + inserted_spaces.len());
        let mut previous = 0;
        let mut columns_by_line = BTreeMap::<u64, Vec<u64>>::new();
        for insertion in &inserted_spaces {
            projected.push_str(&sql[previous..insertion.offset]);
            projected.push(' ');
            previous = insertion.offset;
            columns_by_line
                .entry(insertion.prepared.line)
                .or_default()
                .push(insertion.prepared.column);
        }
        projected.push_str(&sql[previous..]);
        Self {
            sql: Cow::Owned(projected),
            inserted_spaces,
            columns_by_line,
        }
    }

    pub(super) fn sql(&self) -> &str {
        &self.sql
    }

    /// Remove inserted separators and map tokenizer locations back to `sql`.
    pub(super) fn restore(&self, tokens: &mut Vec<TokenWithSpan>) {
        if self.inserted_spaces.is_empty() {
            return;
        }
        let mut next = 0;
        tokens.retain(|token| {
            while self
                .inserted_spaces
                .get(next)
                .is_some_and(|insertion| before(insertion.prepared, token.span.start))
            {
                next += 1;
            }
            let inserted = self
                .inserted_spaces
                .get(next)
                .is_some_and(|insertion| insertion.prepared == token.span.start);
            if inserted {
                next += 1;
            }
            !inserted || !matches!(token.token, Token::Whitespace(Whitespace::Space))
        });
        for token in tokens {
            remap(&mut token.span.start, &self.columns_by_line);
            remap(&mut token.span.end, &self.columns_by_line);
        }
    }

    pub(super) fn restore_error(&self, error: &mut TokenizerError) {
        if !self.inserted_spaces.is_empty() {
            remap(&mut error.location, &self.columns_by_line);
        }
    }
}

fn separator_locations(sql: &str) -> Vec<Insertion> {
    let mut insertions = Vec::new();
    let mut columns_by_line = BTreeMap::<u64, Vec<u64>>::new();
    let mut line = 1;
    let mut column = 1;
    let mut at = 0;
    let mut previous_character = None;
    while at < sql.len() {
        if sql[at..].starts_with("/*") || sql[at..].starts_with("--") {
            let end = skip_comment(sql, at).expect("comment opener");
            if previous_character
                .is_some_and(|character| PostgreSqlDialect {}.is_custom_operator_part(character))
            {
                let prepared_column = column
                    + columns_by_line
                        .get(&line)
                        .map_or(0, |columns| columns.len() as u64);
                let prepared = Location {
                    line,
                    column: prepared_column,
                };
                columns_by_line
                    .entry(line)
                    .or_default()
                    .push(prepared_column);
                insertions.push(Insertion {
                    offset: at,
                    prepared,
                });
            }
            advance(&sql[at..end], &mut line, &mut column);
            previous_character = sql[at..end].chars().next_back();
            at = end;
        } else if let Some(end) = skip_opaque(sql, at) {
            let end = end.min(sql.len());
            advance(&sql[at..end], &mut line, &mut column);
            previous_character = sql[at..end].chars().next_back();
            at = end;
        } else {
            let character = sql[at..].chars().next().expect("valid character boundary");
            advance_char(character, &mut line, &mut column);
            previous_character = Some(character);
            at += character.len_utf8();
        }
    }
    insertions
}

fn advance(text: &str, line: &mut u64, column: &mut u64) {
    for character in text.chars() {
        advance_char(character, line, column);
    }
}

fn advance_char(character: char, line: &mut u64, column: &mut u64) {
    if character == '\n' {
        *line += 1;
        *column = 1;
    } else {
        *column += 1;
    }
}

fn before(left: Location, right: Location) -> bool {
    (left.line, left.column) < (right.line, right.column)
}

fn remap(location: &mut Location, columns_by_line: &BTreeMap<u64, Vec<u64>>) {
    let count = columns_by_line.get(&location.line).map_or(0, |columns| {
        columns.partition_point(|column| *column < location.column) as u64
    });
    location.column -= count;
}

pub(crate) fn tokenize_with_location(
    dialect: &dyn Dialect,
    sql: &str,
) -> Result<Vec<TokenWithSpan>, TokenizerError> {
    let prepared = Prepared::new(sql);
    let mut tokens = match Tokenizer::new(dialect, prepared.sql()).tokenize_with_location() {
        Ok(tokens) => tokens,
        Err(mut error) => {
            prepared.restore_error(&mut error);
            return Err(error);
        }
    };
    prepared.restore(&mut tokens);
    Ok(tokens)
}

pub(crate) fn tokenize(dialect: &dyn Dialect, sql: &str) -> Result<Vec<Token>, TokenizerError> {
    tokenize_with_unescape(dialect, sql, true)
}

pub(crate) fn tokenize_with_unescape(
    dialect: &dyn Dialect,
    sql: &str,
    unescape: bool,
) -> Result<Vec<Token>, TokenizerError> {
    let prepared = Prepared::new(sql);
    let mut tokens = match Tokenizer::new(dialect, prepared.sql())
        .with_unescape(unescape)
        .tokenize_with_location()
    {
        Ok(tokens) => tokens,
        Err(mut error) => {
            prepared.restore_error(&mut error);
            return Err(error);
        }
    };
    prepared.restore(&mut tokens);
    Ok(tokens.into_iter().map(|token| token.token).collect())
}

#[cfg(test)]
mod tests;
