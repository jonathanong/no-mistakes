/// One `LIMIT` / `FETCH FIRST` row cap, at the position of its count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SqlLimitFact {
    pub line: usize,
    pub column: usize,
    pub value: SqlLimitValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlLimitValue {
    /// An integer literal. `FETCH FIRST ROW ONLY` counts as `Literal(1)`.
    Literal(u64),
    /// A placeholder, an expression, or a `FETCH FIRST n PERCENT`.
    Other,
}

/// A limited `SELECT` over one base table, ordered by plain columns of that table: the shape of
/// a page of a keyset walk. Only the WHERE conjuncts decide whether the walk is selective.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlSweepFact {
    pub line: usize,
    /// The column where the table is written, which decides the owning operand of recovered SQL.
    pub column: usize,
    /// The base relation, as written and normalized.
    pub table: String,
    /// The parts of that name, so a quoted dot is not read as a schema qualifier.
    pub table_parts: Vec<String>,
    /// The ORDER BY columns, in order.
    pub order_columns: Vec<String>,
    /// Top-level `AND` conjuncts of WHERE (none when there is no WHERE).
    pub conjuncts: Vec<SqlConjunctFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlConjunctFact {
    /// The conjunct's SQL, lowercased with whitespace collapsed.
    pub text: String,
    /// The columns it compares with a bind parameter as a keyset cursor (`id > $1`,
    /// `(a, b) > ($1, $2)`, or `($1 IS NULL OR id > $1)`); empty for any other predicate.
    pub cursor_columns: Vec<String>,
    /// Which side of those columns the bind bounds; `None` when the conjunct is not a cursor.
    pub cursor_bound: Option<SqlCursorBound>,
    /// The cursor can be switched off by a NULL bind (`($1 IS NULL OR id > $1)`), so it does not
    /// bound the walk unconditionally.
    pub cursor_optional: bool,
    /// `TRUE` or `1 = 1`: a conjunct a query builder seeds a WHERE with, which selects nothing.
    pub constant_true: bool,
}

/// The direction of a cursor comparison: `id > $1` bounds `id` from below, `id < $1` from above.
/// A range (`id >= $1 AND id < $2`) has both, which narrows the walk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlCursorBound {
    Lower,
    Upper,
}
