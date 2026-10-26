//! Monotonic request demand: a general consumer alone does not add locations.
use super::CheckFactPlan;
impl CheckFactPlan {
    fn unclassified_sql_consumer(&self) -> bool {
        self.postgres_bounds
            || self.postgres_fragments
            || self.query_annotation
            || (!self.postgres_conflict_locations && (self.postgres_dml || self.postgres_variants))
    }
    pub(crate) fn record_general_sql_locations(&mut self) {
        self.postgres_general_locations |= self.unclassified_sql_consumer();
    }
    pub(crate) fn wants_full_sql_locations(&self) -> bool {
        self.postgres_exact_locations
            || self.postgres_json_writes
            || (self.postgres_conflict_locations
                && (self.postgres_general_locations || self.unclassified_sql_consumer()))
    }
    pub(super) fn merge_location_demand(&mut self, other: &Self) {
        self.record_general_sql_locations();
        self.postgres_general_locations |=
            other.postgres_general_locations || other.unclassified_sql_consumer();
    }
}
