use super::{SqlBoundItemKind, SqlBoundQuery};

impl SqlBoundQuery {
    /// Opaque FROM/projection sources retain unknown row expansion through query wrappers.
    /// A result LIMIT cannot establish the amount of work inside those sources.
    pub(crate) fn has_opaque_row_inputs(&self) -> bool {
        self.items.iter().any(|item| match &item.kind {
            SqlBoundItemKind::Opaque => true,
            SqlBoundItemKind::Query(query) => query.has_opaque_row_inputs(),
            _ => false,
        })
    }
}
