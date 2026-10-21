mod bounds;
mod debug;
mod iteration;
mod locations;
mod select;
mod writes;
pub use bounds::{
    SqlBareRead, SqlBoundFact, SqlBoundInputMode, SqlBoundItem, SqlBoundItemKind, SqlBoundKind,
    SqlBoundOutput, SqlBoundPin, SqlBoundQuery, SqlPinSource, SqlPossibleTemporary,
    SqlQualifiedRead, SqlQualifiedScope,
};
pub use iteration::{SqlConjunctFact, SqlCursorBound, SqlLimitFact, SqlLimitValue, SqlSweepFact};
pub(crate) use locations::SqlLocationDemand;
pub use locations::{SqlFactPosition, SqlFactSite, SqlVariantLocations};
pub use select::{
    SqlColumnClause, SqlColumnUseFact, SqlCountExistenceFact, SqlExistsSetOpFact,
    SqlRelationPredicateFact, SqlSelectFact, SqlStarProjectionFact,
};
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
#[derive(Clone, PartialEq, Eq, Default)]
pub struct SqlStatementFileFacts {
    /// Request-local provenance for complete embedded alternatives; absent for legacy sources.
    #[doc(hidden)]
    pub variant_locations: Option<SqlVariantLocations>,
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
    /// Source expression dependencies; `unknown` means membership cannot be proven.
    Expressions {
        columns: Vec<String>,
        /// Columns that participate in one expression and must move together.
        groups: Vec<Vec<String>>,
        unknown: bool,
    },
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
