use sqlparser::tokenizer::{Token, TokenWithSpan};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Lexical statement ordinals retain ordering even when lenient ASTs lose spans.
pub(crate) struct Positions(
    BTreeMap<(String, String), VecDeque<Vec<usize>>>,
    BTreeSet<Vec<usize>>,
);

impl Positions {
    pub(crate) fn new(sql: &str) -> Self {
        let mut positions = Self(BTreeMap::new(), BTreeSet::new());
        let tokens = super::super::super::parse::unicode::tokenize_raw_unicode(sql);
        for (ordinal, statement) in tokens
            .split(|token| matches!(token.token, Token::SemiColon))
            .enumerate()
        {
            let code: Vec<_> = statement
                .iter()
                .filter(|token| !matches!(token.token, Token::Whitespace(_)))
                .collect();
            positions.record(&code, &[ordinal]);
            if code.first().is_some_and(|token| word(token, "DO")) {
                for token in &code {
                    if let Token::DollarQuotedString(body) = &token.token {
                        let inner =
                            super::super::super::parse::unicode::tokenize_raw_unicode(&body.value);
                        let mut scope = super::super::dynamic::execution::Scope::default();
                        for (inner_ordinal, statement) in inner
                            .split(|token| matches!(token.token, Token::SemiColon))
                            .enumerate()
                        {
                            let code: Vec<_> = statement
                                .iter()
                                .filter(|token| !matches!(token.token, Token::Whitespace(_)))
                                .collect();
                            if !scope.advance(&code) {
                                positions.1.insert(vec![ordinal, inner_ordinal]);
                            }
                            positions.record(&code, &[ordinal, inner_ordinal]);
                        }
                    }
                }
            }
        }
        positions
    }

    pub(super) fn take(&mut self, kind: &str, table: &str) -> Vec<usize> {
        self.0
            .get_mut(&(kind.to_string(), table.to_string()))
            .and_then(VecDeque::pop_front)
            .unwrap_or_else(|| vec![usize::MAX])
    }

    pub(super) fn executed(&self, order: &[usize]) -> bool {
        !self.1.contains(order)
    }

    fn record(&mut self, tokens: &[&TokenWithSpan], order: &[usize]) {
        for (at, token) in tokens.iter().enumerate() {
            let kind = if word(token, "CREATE") {
                "CREATE"
            } else if word(token, "ALTER") {
                "ALTER"
            } else if word(token, "DROP") {
                "DROP"
            } else {
                continue;
            };
            let mut cursor = at + 1;
            while tokens.get(cursor).is_some_and(|token| {
                ["GLOBAL", "LOCAL", "TEMP", "TEMPORARY", "UNLOGGED"]
                    .iter()
                    .any(|value| word(token, value))
            }) {
                cursor += 1;
            }
            if !tokens.get(cursor).is_some_and(|token| word(token, "TABLE")) {
                continue;
            }
            cursor += 1;
            while tokens.get(cursor).is_some_and(|token| {
                ["IF", "NOT", "EXISTS", "ONLY"]
                    .iter()
                    .any(|value| word(token, value))
            }) {
                cursor += 1;
            }
            loop {
                let (name, next) = object_name(tokens, cursor);
                if name.is_empty() {
                    break;
                }
                self.0
                    .entry((kind.to_string(), name))
                    .or_default()
                    .push_back(order.to_vec());
                cursor = next;
                if kind != "DROP"
                    || !tokens
                        .get(cursor)
                        .is_some_and(|token| matches!(token.token, Token::Comma))
                {
                    break;
                }
                cursor += 1;
            }
        }
    }
}

fn word(token: &TokenWithSpan, expected: &str) -> bool {
    matches!(&token.token, Token::Word(word) if word.quote_style.is_none() && word.value.eq_ignore_ascii_case(expected))
}

fn object_name(tokens: &[&TokenWithSpan], mut cursor: usize) -> (String, usize) {
    let mut parts = Vec::new();
    while let Some(token) = tokens.get(cursor) {
        let Token::Word(word) = &token.token else {
            break;
        };
        parts.push(if word.quote_style.is_some() {
            word.value.clone()
        } else {
            word.value.to_ascii_lowercase()
        });
        cursor += 1;
        if !tokens
            .get(cursor)
            .is_some_and(|token| matches!(token.token, Token::Period))
        {
            break;
        }
        cursor += 1;
    }
    (parts.join("."), cursor)
}
