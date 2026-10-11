use super::EmbeddedSqlSourcePosition;
mod variants;
pub use variants::{EmbeddedSqlVariant, MAX_EMBEDDED_SQL_VARIANTS};

/// One executor call site and its recovered SQL text. For `Dynamic` calls,
/// `sql_text` can be only a verified leading statement rather than complete SQL.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmbeddedSqlCall {
    /// Concrete alternatives for a recoverable Dynamic call; empty for legacy calls.
    pub variants: Vec<EmbeddedSqlVariant>,
    pub line: u32,
    pub callee: String,
    pub sql_text: Option<String>,
    pub kind: EmbeddedSqlKind,
    pub declaration_line: Option<u32>,
    /// Compact physical-line mapping for literal and template recovery.
    pub sql_source_positions: Vec<EmbeddedSqlSourcePosition>,
    /// SQL line and column of each generated interpolation marker, before any user-authored
    /// identifier with the same spelling can be confused for it.
    pub recovered_placeholder_positions: Vec<(u32, u32)>,
}

/// How executed SQL was recovered from TypeScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmbeddedSqlKind {
    #[default]
    Inline,
    ImmutableLocal,
    Composed,
    Dynamic,
}
