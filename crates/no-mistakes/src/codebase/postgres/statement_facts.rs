use std::path::PathBuf;

/// Statement facts for one SQL source (file or embedded call).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlStatementFileFacts {
    pub path: PathBuf,
    pub inserts: Vec<SqlInsertFact>,
    pub selects: Vec<SqlSelectFact>,
    /// One entry per `UPDATE`. Each entry is that statement's relation instances.
    pub updates: Vec<Vec<SqlRelationPredicateFact>>,
    /// One entry per `DELETE`. Each entry is that statement's relation instances.
    pub deletes: Vec<Vec<SqlRelationPredicateFact>>,
    pub triggers: Vec<SqlTriggerFact>,
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
    pub tables: Vec<String>,
    pub predicate_sql: String,
    pub exists_set_operations: Vec<SqlExistsSetOpFact>,
    /// Base-table instances in this SELECT, with columns proven constrained.
    pub relations: Vec<SqlRelationPredicateFact>,
    /// True when this SELECT is the query of `INSERT … SELECT` (including nested
    /// selects inside that query).
    pub in_insert_select: bool,
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
