use super::{SqlBareRead, SqlBoundQuery, SqlQualifiedRead};
use sqlparser::ast::Expr;

/// A conjunct that equates one column or index expression of an item with a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlBoundPin {
    /// Disjunctive alternative owning this pin; None means an unconditional conjunct.
    pub alternative: Option<usize>,
    pub column: String,
    /// Bare column with multiple possible FROM owners; only catalog projection may credit it.
    pub unqualified_candidate: bool,
    /// The canonical table-local expression fixed by this equality, when it is not a column.
    pub expression: Option<Expr>,
    pub source: SqlPinSource,
    /// `IS NOT DISTINCT FROM` also matches NULL; a nullable key needs NULLS NOT DISTINCT.
    pub null_safe: bool,
    /// Bare columns that subqueries in the value read: when one resolves to the enclosing query
    /// the value depends on the row checked, and the pin fixes nothing.
    pub reads: Vec<SqlBareRead>,
    /// Schema-qualified reads retained for catalog identity resolution.
    pub qualified_reads: Vec<SqlQualifiedRead>,
    /// When `source` is one other item, the column of that item this equality names.
    pub peer_column: Option<String>,
    /// Unconditional `=` to one caller value. Lists, arrays, and null-safe matches are not one value.
    pub single_value: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlPinSource {
    /// A literal, placeholder, list, array or scalar subquery: a value the caller sizes.
    Value,
    /// An expression over columns of other items of the same statement (indexes into the items).
    Items(Vec<usize>),
    /// An array expression with unproven caller cardinality, including stored columns
    /// and unknown function results. Indexes retain known source items; this supplies
    /// no finite-key proof, even when an owning row is bounded.
    StoredArray(Vec<usize>),
    /// A syntactically finite array constructor over other items. Every referenced column
    /// must be catalog-proven scalar before its row bound can size the flattened array.
    Array {
        items: Vec<usize>,
        scalar_columns: Vec<(usize, String)>,
        /// Columns accessed only through scalar indexes; their catalog array element must be scalar.
        indexed_columns: Vec<(usize, String)>,
        /// Custom cast targets that require catalog scalar/enum evidence.
        cast_types: Vec<String>,
    },
    /// `IN (SELECT …)` or `= ANY (SELECT …)`: sized by the subquery.
    Query(SqlBoundQuery),
    /// ARRAY(SELECT ...) or a scalar array_agg: input rows and scalar elements need proof.
    ArrayQuery(SqlBoundQuery),
    /// An executed correlated IN/ANY subquery: retain its reads, never grant key credit.
    ReadQuery(SqlBoundQuery),
}
