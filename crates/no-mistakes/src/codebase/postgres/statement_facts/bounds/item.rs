use super::{SqlBoundItem, SqlBoundItemKind};

impl SqlBoundItem {
    pub fn new(
        kind: SqlBoundItemKind,
        alias: Option<String>,
        (line, column): (usize, usize),
    ) -> Self {
        Self {
            kind,
            possible_temporary: None,
            alias,
            alias_explicit: false,
            column_aliases: Vec::new(),
            line,
            column,
            pins: Vec::new(),
            predicate_conjuncts: Vec::new(),
            nonnull_columns: Vec::new(),
            pin_alternatives: 0,
            lateral: false,
            lateral_reads: Vec::new(),
            lateral_qualified_reads: Vec::new(),
        }
    }
}
