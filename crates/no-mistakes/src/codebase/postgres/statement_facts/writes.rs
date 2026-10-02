/// One syntactic INSERT, UPDATE, or MERGE write, independent of any catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlWriteFact {
    pub table: String,
    pub columns: SqlWriteColumns,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlWriteColumns {
    Named(Vec<String>),
    Positional(Option<usize>),
    All,
}
