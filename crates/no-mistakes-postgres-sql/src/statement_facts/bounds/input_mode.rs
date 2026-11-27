/// How inputs determine a query's returned rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SqlBoundInputMode {
    #[default]
    Streaming,
    Blocking,
    /// A data-modifying CTE emits one row per changed target row (item zero).
    Returning,
    /// Source rows of `INSERT ... RETURNING`. A second reference does not prove a join.
    ReturningSource,
    Skipped,
}
