use super::{EmbeddedSqlCall, EmbeddedSqlKind, EmbeddedSqlSourcePosition};
use std::borrow::Cow;

/// Maximum number of concrete statements retained for one executor call.
pub const MAX_EMBEDDED_SQL_VARIANTS: usize = 16;

/// One complete alternative, with its own physical origin and generated binds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedSqlVariant {
    pub sql_text: String,
    /// Physical source byte origin for each byte of `sql_text`.
    pub sql_source_offsets: Vec<u32>,
    pub line: u32,
    pub sql_source_positions: Vec<EmbeddedSqlSourcePosition>,
    pub recovered_placeholder_positions: Vec<(u32, u32)>,
    /// Private append occurrences that contributed SQL to this alternative.
    pub(crate) append_sites: Vec<u32>,
}

impl EmbeddedSqlVariant {
    pub(crate) fn push_unique(values: &mut Vec<Self>, value: Self) {
        let index = values
            .iter()
            .position(|other| {
                other.sql_text == value.sql_text
                    && other.sql_source_offsets == value.sql_source_offsets
                    && other.line == value.line
                    && other.sql_source_positions == value.sql_source_positions
                    && other.recovered_placeholder_positions
                        == value.recovered_placeholder_positions
            })
            .unwrap_or_else(|| {
                values.push(value.clone());
                values.len() - 1
            });
        for site in value.append_sites {
            if !values[index].append_sites.contains(&site) {
                values[index].append_sites.push(site);
            }
        }
    }
}

impl EmbeddedSqlCall {
    pub(crate) fn source_offset_at_sql_offset(&self, offset: usize) -> Option<usize> {
        self.variants
            .first()?
            .sql_source_offsets
            .get(offset)
            .map(|offset| *offset as usize)
    }
    pub(crate) fn source_offset_at_sql_position(
        &self,
        sql: &str,
        line: usize,
        column: usize,
    ) -> Option<usize> {
        let mut current = (1, 1);
        for (offset, character) in sql.char_indices() {
            if current == (line, column) {
                return self.source_offset_at_sql_offset(offset);
            }
            if character == '\n' {
                current = (current.0 + 1, 1);
            } else {
                current.1 += 1;
            }
        }
        None
    }
    /// Project prepared alternatives internally without creating public call sites.
    pub(crate) fn statement_calls(&self) -> impl Iterator<Item = Cow<'_, Self>> + Clone {
        std::iter::once(Cow::Borrowed(self))
            .take(usize::from(self.variants.is_empty()))
            .chain(self.variants.iter().map(|variant| {
                Cow::Owned(Self {
                    line: self.line,
                    callee: self.callee.clone(),
                    sql_text: Some(variant.sql_text.clone()),
                    kind: EmbeddedSqlKind::Composed,
                    declaration_line: Some(variant.line),
                    sql_source_positions: variant.sql_source_positions.clone(),
                    recovered_placeholder_positions: variant
                        .recovered_placeholder_positions
                        .clone(),
                    variants: vec![variant.clone()],
                })
            }))
    }
    pub(crate) fn is_unanalyzable(&self) -> bool {
        self.kind == EmbeddedSqlKind::Dynamic && self.variants.is_empty()
    }
}
