use crate::codebase::postgres::parse::unicode::tokenize_raw_unicode;
use crate::codebase::postgres::types::SqlDeclaredIdentifier;
use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::collections::{BTreeMap, VecDeque};

/// One token pass indexes declarations by kind/name. Dollar-quoted routine
/// bodies are opaque here; recovered bodies get their own location context.
#[derive(Default)]
pub(in crate::codebase::postgres::migration) struct Locations {
    lines: BTreeMap<(String, String), VecDeque<usize>>,
    recursive_views: Vec<SqlDeclaredIdentifier>,
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

    pub(in crate::codebase::postgres::migration) fn unparsed_declarations(
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
        names.extend(self.recursive_views.iter().cloned());
        names.sort_by_key(|name| name.line);
        names
    }

    fn index(&mut self, sql: &str, base_line: usize) {
        let tokens = tokenize_raw_unicode(sql)
            .into_iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_)))
            .collect::<Vec<_>>();
        for (at, token) in tokens.iter().enumerate() {
            self.index_do_body(&tokens, at, base_line);
            if let Some((kind, name, next)) = declaration(&tokens, at) {
                if kind == "view" && recursive_view(&tokens, at) {
                    // sqlparser does not support RECURSIVE VIEW; retain its declared names.
                    self.recursive_views.extend(super::views::names(
                        &tokens,
                        name,
                        next,
                        base_line + token.span.start.line as usize,
                    ));
                    continue;
                }
                self.lines
                    .entry((kind, name))
                    .or_default()
                    .push_back(base_line + token.span.start.line as usize);
            }
        }
    }

    fn index_do_body(&mut self, tokens: &[TokenWithSpan], at: usize, base_line: usize) {
        if !word(tokens.get(at), "DO") {
            return;
        }
        let body_at = at
            + if word(tokens.get(at + 1), "LANGUAGE") {
                3
            } else {
                1
            };
        if let Some(TokenWithSpan {
            token: Token::DollarQuotedString(body),
            span,
        }) = tokens.get(body_at)
        {
            // Lenient parsing recovers direct DO DDL, unlike routine bodies.
            self.index(&body.value, base_line + span.start.line as usize - 1);
        }
    }
}

fn declaration(tokens: &[TokenWithSpan], at: usize) -> Option<(String, String, usize)> {
    let token = tokens.get(at);
    if !word(token, "CREATE") && !word(token, "ALTER") {
        return None;
    }
    let mut object = at + 1;
    while is_modifier(tokens.get(object)) {
        object += 1;
    }
    let Token::Word(kind) = &tokens.get(object)?.token else {
        return None;
    };
    let kind = kind.value.to_ascii_lowercase();
    if kind == "procedure" && word(token, "ALTER") {
        return None;
    }
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
        return None;
    }
    let mut name_at = object + 1;
    while ["IF", "NOT", "EXISTS", "ONLY", "CONCURRENTLY"]
        .iter()
        .any(|keyword| word(tokens.get(name_at), keyword))
    {
        name_at += 1;
    }
    super::procedures::identifier(tokens, name_at).map(|(name, next)| (kind, name, next))
}

fn recursive_view(tokens: &[TokenWithSpan], at: usize) -> bool {
    tokens
        .iter()
        .skip(at + 1)
        .take_while(|token| is_modifier(Some(token)))
        .any(|token| word(Some(token), "RECURSIVE"))
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
        "RECURSIVE",
        "LOCAL",
        "GLOBAL",
    ]
    .iter()
    .any(|keyword| word(token, keyword))
}

pub(super) fn word(token: Option<&TokenWithSpan>, expected: &str) -> bool {
    matches!(token.map(|token| &token.token), Some(Token::Word(word)) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}
