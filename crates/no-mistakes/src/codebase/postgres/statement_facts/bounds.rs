mod kind;
pub use kind::SqlBoundKind;
/// One executed SELECT, UPDATE or DELETE, reduced to its row-count facts.
/// Key uniqueness is determined later against the schema catalog.
#[derive(Clone, PartialEq, Eq)]
pub struct SqlBoundFact {
    /// Owning executed statement, including for independently judged query bodies.
    pub kind: SqlBoundKind,
    /// Independent body obligation, present only when explicitly requested.
    pub body: Option<String>,
    pub line: usize,
    pub column: usize,
    /// Original outer SQL statement position, including WITH or an executing wrapper.
    pub statement_start: Option<(usize, usize)>,
    pub query: SqlBoundQuery,
    pub scalar_reads: Option<SqlScalarReads>,
    /// Qualified enclosing rows fixed for one explicitly LATERAL body invocation.
    pub lateral_qualified_reads: Vec<SqlQualifiedRead>,
    /// For `UPDATE` and `DELETE`: the index in `query.items` of the relation being changed.
    pub target: Option<usize>,
}
/// A query body: its FROM items, and whether it caps its own row count.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlBoundQuery {
    /// A `LIMIT` / `FETCH FIRST`, or a pure aggregate (no `GROUP BY`).
    pub capped: bool,
    /// Input execution and target-row projection policy.
    pub input_mode: SqlBoundInputMode,
    pub items: Vec<SqlBoundItem>,
    /// Output columns in ordinal order; caller-sized values stay finite despite repeated rows.
    pub outputs: Vec<SqlBoundOutput>,
    /// Plain `GROUP BY` of single-value keys. A set-returning `ORDER BY` clears it.
    pub group_constant: bool,
    /// Plain `GROUP BY` keys that stay bounded only when their FROM items are bounded.
    pub group_keys: Vec<SqlGroupKey>,
}

/// One plain `GROUP BY` key. `constant` is a single value; `item` is the FROM item it reads.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlGroupKey {
    pub item: Option<usize>,
    pub constant: bool,
}

/// Syntactic value provenance for one projected column, independent of query row cardinality.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlBoundOutput {
    pub name: Option<String>,
    pub caller_sized: bool,
    /// A syntactically scalar value, or a column whose scalarity the catalog must prove.
    pub scalar: bool,
    pub source_column: Option<(usize, String)>,
    /// A simple array_agg element column; its input rows need an independent bound.
    pub array_aggregate_column: Option<(usize, String)>,
    /// `NULL` or `NULL::type`. An equality on this output matches no row.
    pub constant_null: bool,
    /// Both recursive UNION arms project the same nonrecursive CTE column by ordinal.
    pub recursive_domain: bool,
}

/// One FROM item and the columns or index expressions the statement pins on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlBoundItem {
    pub kind: SqlBoundItemKind,
    /// A request-local temporary relation whose identity needs catalog evidence.
    pub possible_temporary: Option<SqlPossibleTemporary>,
    /// The name columns are qualified by: the alias, else the table's own (bare) name.
    pub alias: Option<String>,
    /// Whether `alias` was written in SQL rather than filled from a relation name.
    pub alias_explicit: bool,
    /// Positional column aliases as written; catalog column names are ambiguous when nonempty.
    pub column_aliases: Vec<String>,
    pub line: usize,
    pub column: usize,
    pub pins: Vec<SqlBoundPin>,
    /// Catalog-independent, table-local conjuncts from WHERE and restricting JOIN ON arms.
    pub predicate_conjuncts: Vec<Expr>,
    /// Conjuncts that hold only inside one OR alternative, in alternative order.
    pub alternative_predicates: Vec<Vec<Expr>>,
    /// Plain columns made non-NULL by strict equality in those same conjuncts.
    pub nonnull_columns: Vec<String>,
    /// Number of disjunctive alternatives; every alternative must complete a unique key.
    pub pin_alternatives: usize,
    /// Alternatives that reject every row of this item, so they need no key.
    pub rejected_alternatives: Vec<usize>,
    /// A certain bare outer read in a `LATERAL` source; its driver ownership is unproven.
    /// Qualified outer reads retain separate scope evidence for driver-dependent bounds.
    pub lateral: bool,
    /// Bare columns of a `LATERAL` source that none of its own relations is known to own: it
    /// reads the items before it when the catalog shows that none of their tables has them.
    pub lateral_reads: Vec<SqlBareRead>,
    /// Schema-qualified reads which may name preceding `LATERAL` items.
    pub lateral_qualified_reads: Vec<SqlQualifiedRead>,
    /// A synthetic transparent wrapper for one `UNION` / `INTERSECT` / `EXCEPT` arm.
    #[doc(hidden)]
    pub set_operation_arm: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlPossibleTemporary {
    /// A three-part `database.pg_temp.name` declaration must match the current database.
    pub database_qualifier: Option<String>,
    /// Explicit search-path schemas before `pg_temp`, in PostgreSQL resolution order.
    pub earlier_schemas: Vec<String>,
    /// An earlier DDL target was unknown: the table may be temporary, so its
    /// catalog keys cannot safely bound another relation.
    pub uncertain_lifetime: bool,
}

/// A bare column that a subquery reads, with the base tables that could own it.
///
/// PostgreSQL resolves a bare column in the innermost query whose relations have it, and
/// otherwise in the query around it. The facts do not know a table's columns, so a subquery
/// reads the enclosing row when the catalog shows that none of `tables` has `column`.
/// Temporary-name projection removes possible temporary candidates because their column
/// layout cannot be proved from a permanent catalog entry; an empty list proves no local owner.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SqlBareRead {
    pub column: String,
    pub tables: Vec<String>,
}

/// A schema-qualified reference which did not match a relation name syntactically.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SqlQualifiedRead {
    /// The qualifier in SQL spelling, preserving quoted identifier boundaries.
    pub qualifier: String,
    /// Candidate scopes ordered from innermost outward.
    pub scopes: Vec<SqlQualifiedScope>,
}

/// Candidate relations in one lexical query scope for a qualified read.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SqlQualifiedScope {
    pub tables: Vec<String>,
    /// The scope contains a relation whose identity cannot be established from its base tables.
    pub unknown: bool,
}

mod item;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlBoundItemKind {
    /// A base relation, as written and normalized.
    Table(String),
    /// A CTE reference, derived table, or set-operation arm.
    Query(SqlBoundQuery),
    /// A `VALUES` list, or a set-returning function sized by its caller-supplied arguments
    /// (`unnest($1)`): it adds no unbounded relation and bounds what is pinned to it.
    Other,
    /// A table function the text does not size (`get_all_accounts()`), or the recursive
    /// reference of a recursive CTE: never reported, and it bounds nothing pinned to it.
    Opaque,
}

mod input_mode;
mod lines;
pub use input_mode::SqlBoundInputMode;
use sqlparser::ast::Expr;
mod debug;
mod row_inputs;

mod pin;
mod scalar;
pub use pin::{SqlBoundPin, SqlPinSource};
pub use scalar::SqlScalarReads;
