use super::{query, Scope};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::Table;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithSpan, Word};

/// Source spellings for TABLE query arms. sqlparser's `Table` AST stores only word values,
/// so a quoted mixed-case name needs the already prepared token stream to retain identity.
#[derive(Default)]
pub(in super::super) struct TableTokenCursor {
    names: Vec<SourceTable>,
    next: usize,
}

#[derive(Clone)]
struct SourceTable {
    schema: Option<String>,
    table: String,
    name: String,
    key: String,
    at: (usize, usize),
}

impl TableTokenCursor {
    pub(in super::super) fn new(tokens: &[TokenWithSpan]) -> Self {
        let words: Vec<_> = tokens
            .iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_)))
            .collect();
        let mut names = Vec::new();
        for (index, token) in words.iter().enumerate() {
            let Token::Word(keyword) = &token.token else {
                continue;
            };
            if keyword.keyword != Keyword::TABLE
                || !query_table_context(words.get(index.wrapping_sub(1)).copied())
            {
                continue;
            }
            let Some(Token::Word(first)) = words.get(index + 1).map(|token| &token.token) else {
                continue;
            };
            let second = match (words.get(index + 2), words.get(index + 3)) {
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
                schema: schema.map(|word| word.value.clone()),
                table: table.value.clone(),
                key: if table.quote_style.is_some() {
                    table.value.clone()
                } else {
                    table.value.to_ascii_lowercase()
                },
                name,
                at: (
                    token.span.start.line as usize,
                    token.span.start.column as usize,
                ),
            });
        }
        Self { names, next: 0 }
    }

    fn take(&mut self, table: &Table) -> Option<SourceTable> {
        let found = self.names[self.next..].iter().position(|source| {
            source.schema.as_deref() == table.schema_name.as_deref()
                && Some(source.table.as_str()) == table.table_name.as_deref()
        })?;
        let source = self.names[self.next + found].clone();
        self.next += found + 1;
        Some(source)
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

/// `TABLE name` is `SELECT * FROM name`: it returns every row of the relation.
pub(super) fn bound(table: &Table, scope: &Scope, at: (usize, usize)) -> SqlBoundQuery {
    let name: Vec<&str> = [table.schema_name.as_deref(), table.table_name.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if name.is_empty() {
        return query::sized_by_itself(at);
    }
    let item = |relation: String, cte_name: &str, at| {
        let cte = table
            .schema_name
            .is_none()
            .then(|| scope.get(cte_name))
            .flatten();
        SqlBoundItem::new(
            match cte {
                Some(bound) => SqlBoundItemKind::Query(bound.clone()),
                None => SqlBoundItemKind::Table(relation),
            },
            None,
            at,
        )
    };
    let recovered = scope
        .table_tokens
        .as_ref()
        .and_then(|cursor| cursor.borrow_mut().take(table));
    let items = if let Some(source) = recovered {
        vec![item(source.name, &source.key, source.at)]
    } else {
        // When source tokens are unavailable, retain both plausible spellings as main did.
        let exact = name
            .iter()
            .map(|part| format!("\"{}\"", part.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(".");
        let folded = name.join(".").to_ascii_lowercase();
        let mut items = vec![item(folded.clone(), &folded, at)];
        if name
            .iter()
            .any(|part| part.to_ascii_lowercase() != *part || part.contains(' '))
        {
            items.push(item(exact, &name.join("."), at));
        }
        items
    };
    SqlBoundQuery {
        capped: false,
        items,
    }
}
