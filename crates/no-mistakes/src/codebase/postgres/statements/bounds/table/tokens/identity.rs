//! Recover TABLE names only when the prepared spellings prove one identity.
use super::{location, SourceSegment, TableTokenCursor, TableTokenIndex};
use sqlparser::{
    ast::{ObjectName, ObjectNamePart, Table},
    tokenizer::{Token, TokenWithSpan},
};

impl TableTokenIndex {
    pub(in crate::codebase::postgres) fn from_iter<'a>(
        tokens: impl IntoIterator<Item = &'a TokenWithSpan>,
    ) -> Self {
        let tokens: Vec<_> = tokens
            .into_iter()
            .filter(|token| !matches!(token.token, Token::Whitespace(_)))
            .collect();
        let Some(first) = tokens.first() else {
            return Self {
                segments: Vec::new(),
            };
        };
        let last = tokens.last().unwrap();
        let cursor = TableTokenCursor::from_iter(tokens.iter().copied());
        Self {
            segments: vec![SourceSegment {
                start: location(first.span.start),
                end: location(last.span.end),
                names: cursor.names,
                operators: cursor.operators,
                froms: cursor.froms,
                depths: cursor.depths,
            }],
        }
    }

    /// A missing AST span cannot distinguish differing quoted spellings. Never guess.
    pub(in crate::codebase::postgres) fn unique_name(&self, table: &Table) -> Option<ObjectName> {
        let mut matches = self
            .segments
            .iter()
            .flat_map(|segment| &segment.names)
            .filter(|source| {
                source.schema.as_deref() == table.schema_name.as_deref()
                    && Some(source.table.as_str()) == table.table_name.as_deref()
            });
        let first = matches.next()?;
        matches.all(|source| source.parts == first.parts).then(|| {
            ObjectName(
                first
                    .parts
                    .iter()
                    .cloned()
                    .map(ObjectNamePart::Identifier)
                    .collect(),
            )
        })
    }
}
