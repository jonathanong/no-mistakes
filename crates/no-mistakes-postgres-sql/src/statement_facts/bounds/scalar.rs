use super::SqlBareRead;

/// Local bare-column candidates and known enclosing relations for an independent scalar read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlScalarReads {
    pub reads: Vec<SqlBareRead>,
    pub outer_tables: Vec<(String, Vec<String>)>,
    pub outer_columns: Vec<String>,
}
