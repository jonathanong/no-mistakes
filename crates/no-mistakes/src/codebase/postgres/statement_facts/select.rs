/// One `SELECT` that names relations in FROM/JOIN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlSelectFact {
    pub line: usize,
    /// Physical relations, excluding visible CTE aliases (all siblings for recursive WITH).
    pub tables: Vec<String>,
    pub predicate_sql: String,
    pub exists_set_operations: Vec<SqlExistsSetOpFact>,
    /// Base-table instances in this SELECT, with columns proven constrained.
    pub relations: Vec<SqlRelationPredicateFact>,
    /// True when this SELECT is the query of `INSERT … SELECT` (including nested
    /// selects inside that query).
    pub in_insert_select: bool,
    /// Lines of `NOT IN (SELECT …)` and `NOT (… IN (SELECT …))`.
    pub not_in_subqueries: Vec<usize>,
    /// Original SQL columns corresponding to `not_in_subqueries`.
    pub not_in_columns: Vec<usize>,
    /// `COUNT(*)` compared with 0 or 1 to test existence.
    ///
    /// Each fact keeps the comparison's line, SQL column, and `negated`
    /// (true when the comparison tests zero rows).
    pub count_existence_checks: Vec<SqlCountExistenceFact>,
    /// `*` / `alias.*` projections over base FROM relations.
    pub star_projections: Vec<SqlStarProjectionFact>,
    /// Bare column references in WHERE, JOIN ON, and ORDER BY.
    pub column_uses: Vec<SqlColumnUseFact>,
}

/// Where a bare column reference was used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlColumnClause {
    Where,
    Join,
    OrderBy,
}

/// A bare column reference resolved to a base relation when possible.
///
/// `table` is empty when the name is unqualified and more than one base
/// relation is in scope. The rule may still assign it from schema facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlColumnUseFact {
    pub table: String,
    pub column: String,
    /// Base relations visible at this expression; None when a CTE or derived relation prevents ownership proof.
    pub candidate_tables: Option<Vec<String>>,
    pub clause: SqlColumnClause,
    pub line: usize,
}

/// A star projection resolved to one base relation.
///
/// `within_function` is set when the star is an argument (`row_to_json(o.*)`).
/// Bare `COUNT(*)` is not recorded. The rule applies `allowWholeRowFunctions`
/// to that name; the statement pass does not know the configured list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlStarProjectionFact {
    pub relation: String,
    pub qualified: bool,
    pub within_function: Option<String>,
    pub line: usize,
}

/// One base-table instance and the columns a predicate constrains on it.
///
/// `constrained_columns` are resolved (qualified, or unqualified in a
/// single-item FROM). `unqualified_columns` are bare names from a
/// multi-relation FROM whose owner a configured catalog may still prove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlRelationPredicateFact {
    pub table: String,
    pub alias: Option<String>,
    pub constrained_columns: Vec<String>,
    pub unqualified_columns: Vec<String>,
    pub line: usize,
}

/// An `EXISTS` whose subquery uses a set operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlExistsSetOpFact {
    pub restricted: bool,
    /// Qualified `table.column` whose qualifier is not a local FROM/WITH name.
    pub correlated: bool,
    pub line: usize,
    /// SQL column of the `EXISTS` keyword. Embedded SQL rebases with this column.
    pub column: usize,
}

/// A scalar `COUNT(*)` comparison testing presence or absence of matching rows.
///
/// `negated` is true when the comparison tests zero matching rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlCountExistenceFact {
    pub line: usize,
    /// Original SQL column used to map embedded clauses to their source line.
    pub column: usize,
    /// True when the comparison tests zero matching rows (NOT EXISTS).
    pub negated: bool,
}
