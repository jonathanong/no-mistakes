use crate::fx::FxHashMap;

/// Internal identity of an occurrence in one prepared SQL projection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[doc(hidden)]
pub enum SqlFactSite {
    Origin,
    Annotation,
    StatementKind(usize),
    Setting(usize),
    Function(usize),
    Write(usize),
    WriteColumn(usize, String),
    Insert(usize),
    Select(usize),
    Relation(usize, usize),
    Star(usize, usize),
    Column(usize, usize),
    NotIn(usize, usize),
    Count(usize, usize),
    Exists(usize, usize),
    UpdateRelation(usize, usize),
    DeleteRelation(usize, usize),
    MutationColumn(usize),
    ReturningStar(usize),
    Offset(usize),
    Limit(usize),
    Sweep(usize),
    Bound(usize),
    BoundRelation(usize, usize),
    Lock(usize),
    Conflict(usize),
}

/// SQL-local token position and its physical embedded-source origin.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub struct SqlFactPosition {
    pub sql_line: usize,
    pub sql_column: usize,
    pub source_line: usize,
    pub source_offset: Option<usize>,
}

/// Provenance is populated only for complete variants, preserving legacy facts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc(hidden)]
pub struct SqlVariantLocations {
    pub original_call_line: u32,
    pub call_index: usize,
    pub variant_index: usize,
    pub positions: FxHashMap<SqlFactSite, SqlFactPosition>,
    pub locking: Vec<super::super::LockingSelectMetadata>,
    pub conflicts: Vec<super::super::SqlConflictInsertFact>,
    pub conflict_error: Option<String>,
    pub(crate) append_sites: Vec<u32>,
    pub(crate) bound_tables: Vec<(usize, String, SqlFactSite)>,
}

impl SqlVariantLocations {
    pub fn bound_table_sites<'a>(
        &'a self,
        bound: usize,
        table: &'a str,
        line: usize,
    ) -> impl Iterator<Item = (SqlFactSite, &'a SqlFactPosition)> + 'a {
        self.bound_tables
            .iter()
            .filter_map(move |(index, name, site)| {
                let parts = super::super::decoded_parts(name);
                let matches =
                    parts.join(".") == table || parts.last().is_some_and(|name| name == table);
                let position = self.position(site.clone())?;
                (*index == bound && matches && position.source_line == line)
                    .then(|| (site.clone(), position))
            })
    }
    pub fn position(&self, site: SqlFactSite) -> Option<&SqlFactPosition> {
        self.positions.get(&site)
    }
    pub(crate) fn insert(&mut self, site: SqlFactSite, line: usize, column: usize) {
        self.positions.insert(
            site,
            SqlFactPosition {
                sql_line: line.max(1),
                sql_column: column.max(1),
                source_line: line.max(1),
                source_offset: None,
            },
        );
    }
}
