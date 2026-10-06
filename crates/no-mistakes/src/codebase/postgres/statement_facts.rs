mod bounds;
mod iteration;
mod writes;
pub use bounds::{
    SqlBareRead, SqlBoundFact, SqlBoundInputMode, SqlBoundItem, SqlBoundItemKind, SqlBoundKind,
    SqlBoundOutput, SqlBoundPin, SqlBoundQuery, SqlPinSource, SqlPossibleTemporary,
    SqlQualifiedRead, SqlQualifiedScope,
};
pub use iteration::{SqlConjunctFact, SqlCursorBound, SqlLimitFact, SqlLimitValue, SqlSweepFact};
use sqlparser::ast::Statement;
use std::collections::BTreeSet;
use std::path::PathBuf;
pub use writes::{SqlWriteColumns, SqlWriteFact};

/// Prepared, source-ordered lifecycle inputs for catalog-dependent SQL bounds.
/// The SQL AST and view reads are collected once; each catalog only projects them.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(hidden)]
pub struct SqlLifecycleFacts {
    pub(crate) raw_bounds: Vec<SqlBoundFact>,
    pub(crate) batches: Vec<SqlLifecycleBatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlLifecycleBatch {
    pub source: Statement,
    pub steps: Vec<SqlLifecycleStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlLifecycleStep {
    pub statement: Statement,
    pub first_bound: usize,
    pub last_bound: usize,
    pub view_reads: Option<SqlViewReads>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SqlViewReads {
    pub query: SqlBoundQuery,
    pub names: BTreeSet<String>,
}

/// Statement facts for one SQL source (file or embedded call).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlStatementFileFacts {
    pub path: PathBuf,
    /// Statement categories shared with schema policy, including static routine DDL.
    pub statement_kinds: Vec<super::types::SqlStatementKind>,
    pub setting_uses: Vec<super::types::SqlSettingUse>,
    pub function_calls: Vec<super::SqlFunctionCallFact>,
    /// Syntactic write targets; catalog resolution happens after preparation.
    pub writes: Vec<SqlWriteFact>,
    pub inserts: Vec<SqlInsertFact>,
    pub selects: Vec<SqlSelectFact>,
    /// One entry per `UPDATE`. Each entry is that statement's relation instances.
    pub updates: Vec<Vec<SqlRelationPredicateFact>>,
    /// One entry per `DELETE`. Each entry is that statement's relation instances.
    pub deletes: Vec<Vec<SqlRelationPredicateFact>>,
    pub triggers: Vec<SqlTriggerFact>,
    /// `RETURNING *` / `RETURNING t.*` on INSERT, UPDATE, and DELETE.
    pub returning_stars: Vec<SqlStarProjectionFact>,
    /// Bare column comparisons on UPDATE/DELETE WHERE and JOIN ON predicates.
    pub mutation_column_uses: Vec<SqlColumnUseFact>,
    /// Executed OFFSET occurrences in source order.
    pub offset_uses: Vec<super::offset::SqlOffsetFact>,
    /// Row-count bounds of each executed SELECT, UPDATE and DELETE.
    pub bounds: Vec<SqlBoundFact>,
    /// Present only when a lifecycle decision needs per-catalog evidence.
    #[doc(hidden)]
    pub lifecycle: Option<SqlLifecycleFacts>,
    /// Every `LIMIT` / `FETCH FIRST` in the executed statements, in source order.
    pub limit_uses: Vec<SqlLimitFact>,
    /// Limited single-table queries ordered by plain columns: pages of a key walk.
    pub sweeps: Vec<SqlSweepFact>,
    pub parse_failed: bool,
    pub insert_keyword_count: usize,
    pub has_top_level_not_exists: bool,
    pub origin_line: usize,
}

/// One executed `INSERT` recovered from PostgreSQL SQL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlInsertFact {
    pub table: String,
    pub line: usize,
    pub executed: bool,
    pub on_conflict: Option<SqlOnConflictFact>,
    pub guarded_select: bool,
    /// INSERT `SET` assignments, or column/value forms from `VALUES` / `SELECT`.
    pub assignments: Vec<SqlAssignmentFact>,
}

/// `ON CONFLICT` action recovered from an INSERT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlOnConflictFact {
    pub action: SqlOnConflictAction,
    pub arbiter: SqlConflictArbiter,
    pub assignments: Vec<SqlAssignmentFact>,
    pub where_proof: SqlConflictWhereProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlOnConflictAction {
    DoNothing,
    DoUpdate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlConflictArbiter {
    Columns(Vec<String>),
    Constraint(String),
    Unknown,
}

/// One SET/VALUES assignment classified without keeping the sqlparser AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlAssignmentFact {
    pub column: String,
    pub form: SqlValueForm,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlValueForm {
    Literal,
    Null,
    Excluded { column: String },
    SelfRef { column: String },
    Coalesce { args: Vec<SqlValueForm> },
    Greatest { args: Vec<SqlValueForm> },
    Least { args: Vec<SqlValueForm> },
    Placeholder,
    Volatile { name: String },
    Subquery,
    Other,
}

/// Conjunctive no-op proofs on `ON CONFLICT … WHERE`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlConflictWhereProof {
    pub distinct_from_excluded: Vec<String>,
    pub null_and_excluded_not_null: Vec<String>,
    pub disjunctive: bool,
}

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

/// One `CREATE TRIGGER`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlTriggerFact {
    pub table: String,
    pub function: String,
    pub period: SqlTriggerPeriod,
    pub for_each_row: bool,
    pub events: Vec<SqlTriggerEvent>,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlTriggerPeriod {
    Before,
    After,
    InsteadOf,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlTriggerEvent {
    Insert,
    Update { columns: Vec<String> },
    Delete,
    Truncate,
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
