use super::SqlColumnMetadata;

/// Ordered table catalog changes, retaining qualified and decoded last-component identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlTableSchemaEvent {
    Create {
        if_not_exists: bool,
        source_order: Vec<usize>,
        table: String,
        unqualified_table: String,
        columns: Vec<SqlColumnMetadata>,
    },
    AddColumn {
        if_not_exists: bool,
        source_order: Vec<usize>,
        table: String,
        unqualified_table: String,
        column: SqlColumnMetadata,
    },
    Drop {
        source_order: Vec<usize>,
        table: String,
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

    pub(crate) fn source_order_mut(&mut self) -> &mut Vec<usize> {
        match self {
            Self::Create { source_order, .. }
            | Self::AddColumn { source_order, .. }
            | Self::Drop { source_order, .. } => source_order,
        }
    }
}
