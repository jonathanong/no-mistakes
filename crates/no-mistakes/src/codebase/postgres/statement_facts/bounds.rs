/// Which statement a bound fact judges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlBoundKind {
    Select,
    Update,
    Delete,
}

/// One executed `SELECT`, `UPDATE` or `DELETE`, reduced to what decides its row count.
///
/// The fact is syntactic: whether a pinned column set is a unique key is decided later,
/// against a schema catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlBoundFact {
    pub kind: SqlBoundKind,
    pub line: usize,
    pub column: usize,
    pub query: SqlBoundQuery,
    /// For `UPDATE` and `DELETE`: the index in `query.items` of the relation being changed.
    pub target: Option<usize>,
}

/// A query body: its FROM items, and whether it caps its own row count.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SqlBoundQuery {
    /// A `LIMIT` / `FETCH FIRST`, or a pure aggregate (no `GROUP BY`).
    pub capped: bool,
    pub items: Vec<SqlBoundItem>,
}

/// One FROM item and the columns the statement pins on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlBoundItem {
    pub kind: SqlBoundItemKind,
    pub alias: Option<String>,
    pub line: usize,
    pub column: usize,
    pub pins: Vec<SqlBoundPin>,
}

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

/// A conjunct that equates one column of an item with a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlBoundPin {
    pub column: String,
    pub source: SqlPinSource,
    /// `IS NOT DISTINCT FROM` also matches NULL, so it picks out one row only on a NOT NULL column.
    pub null_safe: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlPinSource {
    /// A literal, placeholder, list, array or scalar subquery: a value the caller sizes.
    Value,
    /// An expression over columns of other items of the same statement (indexes into the items).
    Items(Vec<usize>),
    /// `IN (SELECT …)` or `= ANY (SELECT …)`: sized by the subquery.
    Query(SqlBoundQuery),
}

impl SqlBoundFact {
    /// Map every position through `map(line, column)`, for SQL embedded in a host file: the
    /// column decides which physical operand of a recovered string owns the line.
    pub fn map_lines(&mut self, map: &impl Fn(usize, usize) -> usize) {
        self.line = map(self.line, self.column);
        self.query.map_lines(map);
    }
}

impl SqlBoundQuery {
    fn map_lines(&mut self, map: &impl Fn(usize, usize) -> usize) {
        for item in &mut self.items {
            item.line = map(item.line, item.column);
            if let SqlBoundItemKind::Query(query) = &mut item.kind {
                query.map_lines(map);
            }
            for pin in &mut item.pins {
                if let SqlPinSource::Query(query) = &mut pin.source {
                    query.map_lines(map);
                }
            }
        }
    }
}
