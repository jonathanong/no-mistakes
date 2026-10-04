use super::{SourceDepth, SourceOperator, SourceTable, TableTokenCursor};
use sqlparser::ast::Ident;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};

impl TableTokenCursor {
    pub(in super::super) fn new(tokens: &[TokenWithSpan]) -> Self {
        Self::from_iter(tokens)
    }

    pub(super) fn from_iter<'a>(tokens: impl IntoIterator<Item = &'a TokenWithSpan>) -> Self {
        let words: Vec<_> = tokens
            .into_iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_)))
            .collect();
        let mut names = Vec::new();
        let mut operators = Vec::new();
        let mut froms = Vec::new();
        let mut depths = Vec::new();
        let mut depth: usize = 0;
        for (index, token) in words.iter().enumerate() {
            if token.token == Token::RParen {
                depth = depth.saturating_sub(1);
            }
            let at = (
                token.span.start.line as usize,
                token.span.start.column as usize,
            );
            depths.push(SourceDepth { at, depth });
            if token.token == Token::LParen {
                depth += 1;
            }
            let Token::Word(keyword) = &token.token else {
                continue;
            };
            if matches!(
                keyword.keyword,
                Keyword::UNION | Keyword::INTERSECT | Keyword::EXCEPT
            ) {
                operators.push(SourceOperator { at, depth });
            }
            if keyword.keyword == Keyword::FROM {
                froms.push(SourceOperator { at, depth });
            }
            if keyword.keyword != Keyword::TABLE
                || !query_table_context(words.get(index.wrapping_sub(1)).copied())
            {
                continue;
            }
            let first_index = index
                + if matches!(words.get(index + 1).map(|token| &token.token), Some(Token::Word(word)) if word.quote_style.is_none() && word.keyword == Keyword::ONLY)
                {
                    2
                } else {
                    1
                };
            let Some(Token::Word(first)) = words.get(first_index).map(|token| &token.token) else {
                continue;
            };
            let second = match (words.get(first_index + 1), words.get(first_index + 2)) {
                (Some(dot), Some(name)) if matches!(dot.token, Token::Period) => {
                    let Token::Word(word) = &name.token else {
                        continue;
                    };
                    Some(word)
                }
                _ => None,
            };
            let (schema, table) = if let Some(second) = second {
                (Some(first), second)
            } else {
                (None, first)
            };
            let name = [schema, Some(table)]
                .into_iter()
                .flatten()
                .map(sql_name)
                .collect::<Vec<_>>()
                .join(".");
            names.push(SourceTable {
                parts: [schema, Some(table)]
                    .into_iter()
                    .flatten()
                    .map(|word| match word.quote_style {
                        Some(quote) => Ident::with_quote(quote, &word.value),
                        None => Ident::new(&word.value),
                    })
                    .collect(),
                schema: schema.map(|word| word.value.clone()),
                table: table.value.clone(),
                key: if table.quote_style.is_some() {
                    table.value.clone()
                } else {
                    table.value.to_ascii_lowercase()
                },
                name,
                at,
                depth,
            });
        }
        Self {
            names,
            operators,
            froms,
            depths,
            next: 0,
            last_at: None,
            last_depth: 0,
            last_operator: None,
        }
    }
}

fn query_table_context(previous: Option<&TokenWithSpan>) -> bool {
    match previous.map(|token| &token.token) {
        None | Some(Token::SemiColon | Token::LParen | Token::RParen) => true,
        Some(Token::Word(word)) => matches!(
            word.value.to_ascii_uppercase().as_str(),
            "ALL" | "DISTINCT" | "UNION" | "INTERSECT" | "EXCEPT" | "AS"
        ),
        _ => false,
    }
}

fn sql_name(word: &Word) -> String {
    if word.quote_style == Some('"') {
        format!("\"{}\"", word.value.replace('"', "\"\""))
    } else {
        word.value.to_ascii_lowercase()
    }
}
