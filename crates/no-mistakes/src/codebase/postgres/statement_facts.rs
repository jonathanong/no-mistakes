use std::path::PathBuf;

/// Statement facts for one SQL source (file or embedded call).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlStatementFileFacts {
    pub path: PathBuf,
    pub inserts: Vec<SqlInsertFact>,
    pub selects: Vec<SqlSelectFact>,
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
}

/// An `EXISTS` whose subquery uses a set operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlExistsSetOpFact {
    pub restricted: bool,
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
