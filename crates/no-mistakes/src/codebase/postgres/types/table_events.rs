use super::SqlColumnMetadata;

/// Ordered table catalog changes, retaining qualified and decoded last-component identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlTableSchemaEvent {
    Create {
        temporary: bool,
        if_not_exists: bool,
        /// False for `AS SELECT`, `LIKE`, `CLONE`, `INHERITS`, and partitions, whose
        /// physical column order is not fully enumerated by `columns`.
        columns_complete: bool,
        source_order: Vec<usize>,
        /// 1-based physical line where the statement starts (0 when unknown).
        line: usize,
        table: String,
        relation_key: String,
        unqualified_table: String,
        columns: Vec<SqlColumnMetadata>,
    },
    AddColumn {
        table_if_exists: bool,
        if_not_exists: bool,
        source_order: Vec<usize>,
        /// 1-based physical line where the statement starts (0 when unknown).
        line: usize,
        table: String,
        relation_key: String,
        unqualified_table: String,
        column: SqlColumnMetadata,
    },
    Drop {
        source_order: Vec<usize>,
        /// 1-based physical line where the statement starts (0 when unknown).
        line: usize,
        table: String,
        relation_key: String,
        unqualified_table: String,
    },
}

impl SqlTableSchemaEvent {
    pub(crate) fn source_order(&self) -> &[usize] {
        match self {
            Self::Create { source_order, .. }
            | Self::AddColumn { source_order, .. }
            | Self::Drop { source_order, .. } => source_order,
        }
    }

    pub(crate) fn line(&self) -> usize {
        match self {
            Self::Create { line, .. } | Self::AddColumn { line, .. } | Self::Drop { line, .. } => {
                *line
            }
        }
    }

    pub(crate) fn line_mut(&mut self) -> &mut usize {
        match self {
            Self::Create { line, .. } | Self::AddColumn { line, .. } | Self::Drop { line, .. } => {
                line
            }
        }
    }

    pub(crate) fn source_order_mut(&mut self) -> &mut Vec<usize> {
        match self {
            Self::Create { source_order, .. }
            | Self::AddColumn { source_order, .. }
            | Self::Drop { source_order, .. } => source_order,
        }
    }
}
