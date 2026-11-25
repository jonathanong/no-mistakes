use super::{names, SchemaCatalog};

impl SchemaCatalog {
    pub(crate) fn has_descendants(&self, relation: &str) -> Option<bool> {
        let table = self
            .relation(relation)
            .filter(|table| self.same_relation(relation, &table.name))?;
        self.descendant_presence
            .get(&names::normalize_table_name(&table.name))
            .copied()
    }

    /// Whether two SQL names have the same catalog identity. Equating a bare name with a
    /// schema-qualified one requires explicit evidence for the selected schema.
    pub(crate) fn same_relation(&self, left: &str, right: &str) -> bool {
        let left_normalized = names::normalize_table_name(left);
        let right_normalized = names::normalize_table_name(right);
        if left_normalized == right_normalized {
            return true;
        }
        let (left_schema, _) = names::split_name(left);
        let (right_schema, _) = names::split_name(right);
        if [left_schema.as_deref(), right_schema.as_deref()]
            .into_iter()
            .flatten()
            .any(|schema| self.schema.as_deref() != Some(schema))
        {
            return false;
        }
        self.relation(left)
            .zip(self.relation(right))
            .is_some_and(|(left, right)| left.name == right.name)
    }
}
