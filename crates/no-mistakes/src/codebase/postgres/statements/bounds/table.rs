use super::{query, Scope};
use crate::codebase::postgres::statements::{SqlBoundItem, SqlBoundItemKind, SqlBoundQuery};
use sqlparser::ast::Table;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Location, Token, TokenWithSpan, Word};

/// Query-arm spellings grouped by their original top-level SQL statement. A skipped DDL
/// statement must never supply a name to a later analyzed TABLE arm.
pub(in super::super) struct TableTokenIndex {
    segments: Vec<SourceSegment>,
}

struct SourceSegment {
    start: (u64, u64),
    end: (u64, u64),
    names: Vec<SourceTable>,
    operators: Vec<SourceOperator>,
    depths: Vec<SourceDepth>,
}

impl TableTokenIndex {
    pub(in super::super) fn new(tokens: &[TokenWithSpan]) -> Self {
        let segments = tokens
            .split(|token| token.token == Token::SemiColon)
            .filter_map(|segment| {
                let first = segment
                    .iter()
                    .find(|token| !matches!(token.token, Token::Whitespace(_)))?;
                let last = segment
                    .iter()
                    .rfind(|token| !matches!(token.token, Token::Whitespace(_)))?;
                let cursor = TableTokenCursor::new(segment);
                Some(SourceSegment {
                    start: location(first.span.start),
                    end: location(last.span.end),
                    names: cursor.names,
                    operators: cursor.operators,
                    depths: cursor.depths,
                })
            })
            .collect();
        Self { segments }
    }

    pub(in super::super) fn cursor_at(&self, start: Location) -> TableTokenCursor {
        let start = location(start);
        let Some(index) = self
            .segments
            .partition_point(|segment| segment.start <= start)
            .checked_sub(1)
        else {
            return TableTokenCursor::default();
        };
        let segment = &self.segments[index];
        if start > segment.end {
            return TableTokenCursor::default();
        }
        TableTokenCursor {
            names: segment.names.clone(),
            operators: segment.operators.clone(),
            depths: segment.depths.clone(),
            next: 0,
            last_at: None,
            last_depth: 0,
            last_operator: None,
        }
    }
}

fn location(at: Location) -> (u64, u64) {
    (at.line, at.column)
}

/// Source spellings for TABLE query arms. sqlparser's `Table` AST stores only word values,
/// so a quoted mixed-case name needs the already prepared token stream to retain identity.
#[derive(Default)]
pub(in super::super) struct TableTokenCursor {
    names: Vec<SourceTable>,
    operators: Vec<SourceOperator>,
    depths: Vec<SourceDepth>,
    next: usize,
    last_at: Option<(usize, usize)>,
    last_depth: usize,
    last_operator: Option<(usize, usize)>,
}

#[derive(Clone)]
struct SourceOperator {
    at: (usize, usize),
    depth: usize,
}

#[derive(Clone)]
struct SourceDepth {
    at: (usize, usize),
    depth: usize,
}

#[derive(Clone)]
struct SourceTable {
    schema: Option<String>,
    table: String,
    name: String,
    key: String,
    at: (usize, usize),
    depth: usize,
}

impl TableTokenCursor {
    pub(in super::super) fn new(tokens: &[TokenWithSpan]) -> Self {
        let words: Vec<_> = tokens
            .iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_)))
            .collect();
        let mut names = Vec::new();
        let mut operators = Vec::new();
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
                at,
                depth,
            });
        }
        Self {
            names,
            operators,
            depths,
            next: 0,
            last_at: None,
            last_depth: 0,
            last_operator: None,
        }
    }

    fn take(&mut self, table: &Table) -> Option<SourceTable> {
        let found = self.names[self.next..].iter().position(|source| {
            source.schema.as_deref() == table.schema_name.as_deref()
                && Some(source.table.as_str()) == table.table_name.as_deref()
        })?;
        let source = self.names[self.next + found].clone();
        self.next += found + 1;
        self.last_at = Some(source.at);
        self.last_depth = source.depth;
        Some(source)
    }

    pub(super) fn advance_to_right_arm(&mut self, left_start: Location) {
        let start = (left_start.line as usize, left_start.column as usize);
        let (from, max_depth) = if start == (0, 0) {
            (self.last_at.unwrap_or(start), self.last_depth)
        } else {
            let depth = self
                .depths
                .iter()
                .rfind(|source| source.at <= start)
                .map_or(0, |source| source.depth);
            (start.max(self.last_at.unwrap_or(start)), depth)
        };
        let from = from.max(self.last_operator.unwrap_or(from));
        let Some(operator) = self
            .operators
            .iter()
            .find(|operator| operator.at > from && operator.depth <= max_depth)
        else {
            return;
        };
        self.last_operator = Some(operator.at);
        while self
            .names
            .get(self.next)
            .is_some_and(|source| source.at < operator.at)
        {
            self.next += 1;
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
