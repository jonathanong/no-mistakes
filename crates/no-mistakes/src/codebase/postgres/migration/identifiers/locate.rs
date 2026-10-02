use crate::codebase::postgres::parse::unicode::tokenize_raw_unicode;
use crate::codebase::postgres::types::SqlDeclaredIdentifier;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::collections::{BTreeMap, VecDeque};

/// One token pass indexes declarations by kind/name. Dollar-quoted routine
/// bodies are opaque here; recovered bodies get their own location context.
#[derive(Default)]
pub(in crate::codebase::postgres::migration) struct Locations {
    lines: BTreeMap<(String, String), VecDeque<usize>>,
}

impl Locations {
    pub(in crate::codebase::postgres::migration) fn new(sql: &str) -> Self {
        let mut locations = Self::default();
        locations.index(sql, 0);
        locations
    }

    pub(super) fn take(&mut self, kind: &str, name: &str) -> Option<usize> {
        self.lines
            .get_mut(&(kind.to_owned(), name.to_owned()))?
            .pop_front()
    }

    pub(in crate::codebase::postgres::migration) fn procedures(
        &self,
    ) -> Vec<SqlDeclaredIdentifier> {
        let mut names = self
            .lines
            .iter()
            .filter(|((kind, _), _)| kind == "procedure")
            .flat_map(|((_, name), lines)| {
                lines.iter().map(|line| SqlDeclaredIdentifier {
                    name: name.clone(),
                    line: *line,
                })
            })
            .collect::<Vec<_>>();
        names.sort_by_key(|name| name.line);
        names
    }

    fn index(&mut self, sql: &str, base_line: usize) {
        let tokens = tokenize_raw_unicode(sql)
            .into_iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_)))
            .collect::<Vec<_>>();
        for (at, token) in tokens.iter().enumerate() {
            if word(Some(token), "DO") {
                if let Some(TokenWithSpan {
                    token: Token::DollarQuotedString(body),
                    span,
                }) = tokens.get(at + 1)
                {
                    // Lenient parsing already recovers direct DO DDL, unlike routine bodies.
                    self.index(&body.value, base_line + span.start.line as usize - 1);
                }
            }
            if !word(Some(token), "CREATE") && !word(Some(token), "ALTER") {
                continue;
            }
            let mut object = at + 1;
            while is_modifier(tokens.get(object)) {
                object += 1;
            }
            let Some(TokenWithSpan {
                token: Token::Word(kind),
                ..
            }) = tokens.get(object)
            else {
                continue;
            };
            let kind = kind.value.to_ascii_lowercase();
            if ![
                "table",
                "index",
                "view",
                "trigger",
                "function",
                "procedure",
                "type",
            ]
            .contains(&kind.as_str())
            {
                continue;
            }
            let mut name_at = object + 1;
            while ["IF", "NOT", "EXISTS", "ONLY", "CONCURRENTLY"]
                .iter()
                .any(|keyword| word(tokens.get(name_at), keyword))
            {
                name_at += 1;
            }
            if let Some((name, _)) = super::procedures::identifier(&tokens, name_at) {
                self.lines
                    .entry((kind, name))
                    .or_default()
                    .push_back(base_line + token.span.start.line as usize);
            }
        }
    }
}

fn is_modifier(token: Option<&TokenWithSpan>) -> bool {
    [
        "OR",
        "REPLACE",
        "TEMP",
        "TEMPORARY",
        "UNLOGGED",
        "CONSTRAINT",
        "UNIQUE",
        "MATERIALIZED",
        "LOCAL",
        "GLOBAL",
    ]
    .iter()
    .any(|keyword| word(token, keyword))
}

pub(super) fn word(token: Option<&TokenWithSpan>, expected: &str) -> bool {
    matches!(token.map(|token| &token.token), Some(Token::Word(word)) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
