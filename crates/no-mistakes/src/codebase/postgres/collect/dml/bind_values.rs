use crate::codebase::postgres::{embedded::EmbeddedSqlCall, statements::BindValues};
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn collect(call: &EmbeddedSqlCall, file: &std::path::Path) -> BindValues {
    if call.bound_value_origins.is_empty()
        || call.bound_value_sql.as_deref() != call.sql_text.as_deref()
    {
        return Arc::default();
    }
    let mut values = BTreeMap::new();
    for &(line, column, id) in &call.bound_value_origins {
        if !call
            .recovered_placeholder_positions
            .contains(&(line, column))
        {
            continue;
        }
        let source = call
            .sql_source_positions
            .partition_point(|position| (position.sql_line, position.sql_column) <= (line, column))
            .checked_sub(1)
            .and_then(|index| call.sql_source_positions[index].source_path.as_deref());
        if !source.is_some_and(|source| source != file) {
            values.insert((line, column), id);
        }
    }
    Arc::new(values)
}

#[cfg(test)]
#[path = "bind_values_tests.rs"]
mod tests;
