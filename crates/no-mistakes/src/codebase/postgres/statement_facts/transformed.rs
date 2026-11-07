/// Predicate context for a cast or function applied to a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlTransformedColumnClause {
    Where,
    Join,
    Having,
}

/// A syntactic column reference below a cast or function, resolved against base relations.
/// An empty table needs catalog disambiguation; absent candidates mean unknown scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlTransformedColumnUseFact {
    pub table: String,
    pub column: String,
    pub candidate_tables: Option<Vec<String>>,
    pub clause: SqlTransformedColumnClause,
    pub line: usize,
    pub column_position: usize,
    pub transformation: String,
    pub expression_site: (usize, usize),
    pub driving_probe: Option<SqlTransformedColumnProbeFact>,
}

/// Bare probe key whose catalog type must be proven before exempting driving transforms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlTransformedColumnProbeFact {
    pub table: String,
    pub column: String,
    pub candidate_tables: Option<Vec<String>>,
}
