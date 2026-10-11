pub(super) fn interpolated_relation_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: multi-row FOR UPDATE locks a relation whose name is interpolated, so its schema-catalog key order cannot be checked; write the relation name literally, use SKIP LOCKED, or add a `{directive}` comment"
    )
}

pub(super) fn canonical_order_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: multi-row FOR UPDATE ORDER BY must begin with a valid schema-catalog unique-key order; add the catalog key prefix, use SKIP LOCKED, or add a `{directive}` comment"
    )
}

pub(super) fn lock_ordering_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: multi-row FOR UPDATE without ORDER BY or SKIP LOCKED can deadlock (ABBA); add ORDER BY, use SKIP LOCKED, or add a `{directive}` comment"
    )
}

pub(super) fn unparseable_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: keep FOR UPDATE SQL parseable so lock ordering can be checked, or add a `{directive}` comment"
    )
}

pub(super) fn unanalyzable_message(file: &str, line: u32, directive: &str) -> String {
    format!(
        "{file}:{line}: executed SQL is not statically recoverable, so a FOR UPDATE lock and its row order cannot be checked for ABBA deadlocks; pass a SQL literal or trusted tagged template, add a `{directive}` comment, or set unanalyzableSql: ignore"
    )
}
