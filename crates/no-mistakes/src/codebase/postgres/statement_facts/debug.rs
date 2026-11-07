use super::{SqlSelectFact, SqlStatementFileFacts};
use std::fmt::{Debug, Formatter, Result};

impl Debug for SqlStatementFileFacts {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        let mut out = formatter.debug_struct("SqlStatementFileFacts");
        out.field("path", &self.path)
            .field("statement_kinds", &self.statement_kinds)
            .field("setting_uses", &self.setting_uses)
            .field("function_calls", &self.function_calls)
            .field("writes", &self.writes)
            .field("inserts", &self.inserts)
            .field("selects", &self.selects)
            .field("updates", &self.updates)
            .field("deletes", &self.deletes)
            .field("triggers", &self.triggers)
            .field("returning_stars", &self.returning_stars)
            .field("mutation_column_uses", &self.mutation_column_uses);
        // Empty additive facts must preserve the legacy projection bytes.
        if !self.transformed_column_uses.is_empty() {
            out.field("transformed_column_uses", &self.transformed_column_uses);
        }
        out.field("offset_uses", &self.offset_uses)
            .field("bounds", &self.bounds)
            .field("lifecycle", &self.lifecycle)
            .field("limit_uses", &self.limit_uses)
            .field("sweeps", &self.sweeps)
            .field("parse_failed", &self.parse_failed)
            .field("insert_keyword_count", &self.insert_keyword_count)
            .field("has_top_level_not_exists", &self.has_top_level_not_exists)
            .field("origin_line", &self.origin_line);
        if !self.parse_failures.is_empty() {
            out.field("parse_failures", &self.parse_failures);
        }
        if let Some(locations) = &self.variant_locations {
            out.field("variant_locations", locations);
        }
        out.finish()
    }
}

#[cfg(test)]
mod tests;

impl Debug for SqlSelectFact {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("SqlSelectFact")
            .field("line", &self.line)
            .field("tables", &self.tables)
            .field("predicate_sql", &self.predicate_sql)
            .field("exists_set_operations", &self.exists_set_operations)
            .field("relations", &self.relations)
            .field("in_insert_select", &self.in_insert_select)
            .field("not_in_subqueries", &self.not_in_subqueries)
            .field("not_in_columns", &self.not_in_columns)
            .field("count_existence_checks", &self.count_existence_checks)
            .field("star_projections", &self.star_projections)
            .field("column_uses", &self.column_uses)
            .finish()
    }
}
