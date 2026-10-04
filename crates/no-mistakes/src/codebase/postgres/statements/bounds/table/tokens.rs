use sqlparser::ast::Table;
use sqlparser::tokenizer::{Location, Token, TokenWithSpan};

mod scan;
#[cfg(test)]
mod tests;

/// Query-arm spellings grouped by their original top-level SQL statement. A skipped DDL
/// statement must never supply a name to a later analyzed TABLE arm.
pub(in super::super::super) struct TableTokenIndex {
    segments: Vec<SourceSegment>,
}

struct SourceSegment {
    start: (u64, u64),
    end: (u64, u64),
    names: Vec<SourceTable>,
    operators: Vec<SourceOperator>,
    froms: Vec<SourceOperator>,
    depths: Vec<SourceDepth>,
}

impl TableTokenIndex {
    pub(in super::super::super) fn new(tokens: &[TokenWithSpan]) -> Self {
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
                    froms: cursor.froms,
                    depths: cursor.depths,
                })
            })
            .collect();
        Self { segments }
    }

    pub(in super::super::super) fn cursor_at(&self, start: Location) -> TableTokenCursor {
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
            froms: segment.froms.clone(),
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

fn first_after<T, K: Ord + Copy>(items: &[T], at: K, mut position: impl FnMut(&T) -> K) -> usize {
    items.partition_point(|item| position(item) <= at)
}

/// Source spellings for TABLE query arms. sqlparser's `Table` AST stores only word values,
/// so a quoted mixed-case name needs the already prepared token stream to retain identity.
#[derive(Default)]
pub(in super::super::super) struct TableTokenCursor {
    names: Vec<SourceTable>,
    operators: Vec<SourceOperator>,
    froms: Vec<SourceOperator>,
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
pub(super) struct SourceTable {
    schema: Option<String>,
    table: String,
    pub(super) name: String,
    pub(super) key: String,
    pub(super) at: (usize, usize),
    depth: usize,
}

impl TableTokenCursor {
    /// Ignore TABLE spellings in a SELECT projection before traversing its FROM sources.
    /// A scalar query can contain its own FROM, so match only the SELECT's nesting depth.
    pub(in super::super::super) fn advance_to_from(&mut self, select_start: Location) {
        let start = (select_start.line as usize, select_start.column as usize);
        let depth_before = first_after(&self.depths, start, |source| source.at);
        let depth = depth_before
            .checked_sub(1)
            .map_or(0, |index| self.depths[index].depth);
        let first_from = first_after(&self.froms, start, |from| from.at);
        let Some(from) = self.froms[first_from..]
            .iter()
            .find(|from| from.depth == depth)
        else {
            return;
        };
        while self
            .names
            .get(self.next)
            .is_some_and(|name| name.at < from.at)
        {
            self.next += 1;
        }
    }

    pub(super) fn take(&mut self, table: &Table) -> Option<SourceTable> {
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

    pub(in super::super::super) fn advance_to_right_arm(&mut self, left_start: Location) {
        let start = (left_start.line as usize, left_start.column as usize);
        let (from, max_depth) = if start == (0, 0) {
            (self.last_at.unwrap_or(start), self.last_depth)
        } else {
            let depth_before = first_after(&self.depths, start, |source| source.at);
            let depth = depth_before
                .checked_sub(1)
                .map_or(0, |index| self.depths[index].depth);
            (start.max(self.last_at.unwrap_or(start)), depth)
        };
        let from = from.max(self.last_operator.unwrap_or(from));
        let first_operator = first_after(&self.operators, from, |operator| operator.at);
        let Some(operator) = self.operators[first_operator..]
            .iter()
            .find(|operator| operator.depth <= max_depth)
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
