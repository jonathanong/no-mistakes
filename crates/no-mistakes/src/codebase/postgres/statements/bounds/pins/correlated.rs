mod columns;
mod ctes;
mod scalar_arrays;
mod scope;
use scope::Scope;
mod sources;
mod visitor;
use super::super::super::value::{is_placeholder_ident_at, PlaceholderPositions};
use super::super::items::sql_name;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use crate::codebase::postgres::statements::SqlBareRead;
use crate::fx::FxHashMap;
use columns::output_names;
pub(in super::super) use columns::projection_columns;
use sqlparser::ast::{ObjectName, Query, SetExpr, Visit};
use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

/// What a subquery reads of the query around it.
#[derive(Default)]
pub(in super::super) struct Reads {
    /// It reads a column of the enclosing query whatever the catalog says.
    pub(in super::super) certain: bool,
    /// Bare columns that read the enclosing query unless a catalog table owns them.
    pub(in super::super) bare: Vec<SqlBareRead>,
}

/// Whether `query` reads a column of a relation outside it, so the rows it returns depend on
/// the row being checked and its size says nothing about how many rows match.
///
/// `outer` holds the names the enclosing FROM items answer to, and `ctes` the CTE names in scope
/// (a relation with one of those names is not a base table). References are resolved one query
/// level at a time, innermost first: a qualified column is the level's own when one of its
/// relations has that name, and otherwise belongs to the level above. A bare column belongs to
/// the first level with a relation that has it. Without the catalog that is known only for a
/// level with no relation (it reads the level above) or one with a relation other than a base
/// table (assumed to have it); for base tables the candidates are returned for the catalog.
pub(in super::super) fn reads_outer_rows(
    query: &Query,
    outer: &BTreeSet<String>,
    ctes: &BTreeMap<String, Option<BTreeSet<String>>>,
    positions: PlaceholderPositions<'_>,
) -> Reads {
    let mut scan = Scan {
        ctes: ctes.clone(),
        positions: positions.map(<[(u32, u32)]>::to_vec),
        ..Scan::default()
    };
    let _ = query.visit(&mut scan);
    let certain = scan
        .unresolved
        .iter()
        .any(|qualifier| outer.contains(qualifier))
        || scan.reads.iter().any(|read| read.tables.is_empty());
    let mut bare: Vec<SqlBareRead> = scan
        .reads
        .into_iter()
        .filter(|read| !read.tables.is_empty())
        .collect();
    bare.sort();
    bare.dedup();
    Reads { certain, bare }
}

/// What one query level mentions, until its relations are all known.
#[derive(Default)]
struct Frame {
    /// Restore the enclosing query's CTE names when this scope unwinds.
    previous_ctes: BTreeMap<String, Option<BTreeSet<String>>>,
    scope: Scope,
    /// A set operation's SELECT arms have independent aliases and projected columns.
    split_selects: bool,
    /// This frame belongs to one SELECT arm and must finish at `post_visit_select`.
    select_frame: bool,
    /// Derived queries resolve against preceding sources, never their own output.
    enclosing: Option<Scope>,
    escaping_qualifiers: Vec<String>,
    escaping_reads: Vec<SqlBareRead>,
    qualifiers: Vec<String>,
    /// How often each bare name occurs, and how often as a whole `ORDER BY` or `GROUP BY` item,
    /// where it can name an output column instead of a relation's column.
    bare: BTreeMap<String, usize>,
    labels: BTreeMap<String, usize>,
    /// Bare reads of the levels below that none of them owns.
    reads: Vec<SqlBareRead>,
}

#[derive(Default)]
struct Scan {
    stack: Vec<Frame>,
    /// AST identity is used only during this visitor run; no sources are reparsed.
    derived_scopes: BTreeMap<usize, Scope>,
    ctes: BTreeMap<String, Option<BTreeSet<String>>>,
    pending_ctes: FxHashMap<usize, (String, Option<BTreeSet<String>>)>,
    /// Qualifiers and bare reads that no level of the query resolved.
    unresolved: Vec<String>,
    reads: Vec<SqlBareRead>,
    positions: Option<Vec<(u32, u32)>>,
}

impl Scan {
    fn is_cte(&self, name: &ObjectName) -> bool {
        name.0.len() == 1
            && object_name_ident(name)
                .is_some_and(|ident| self.ctes.contains_key(&ident_key(ident)))
    }
}

impl Scan {
    fn finish_frame(&mut self, query: &Query) -> ControlFlow<()> {
        let previous_ctes = self
            .stack
            .last()
            .map(|frame| frame.previous_ctes.clone())
            .unwrap_or_default();
        let result = self.finish_frame_without_query();
        self.ctes = previous_ctes;
        self.complete_cte(query);
        result
    }

    fn finish_frame_without_query(&mut self) -> ControlFlow<()> {
        let frame = self.stack.pop().unwrap_or_default();
        let mut up: Vec<String> = frame
            .qualifiers
            .into_iter()
            .filter(|qualifier| !frame.scope.relations.contains(qualifier))
            .collect();
        // A bare name that is a relation's own is a whole-row reference, not a column.
        let own = frame.bare.iter().filter(|(name, count)| {
            **count > frame.labels.get(*name).copied().unwrap_or(0)
                && !frame.scope.relations.contains(name)
        });
        let mut reads: Vec<SqlBareRead> = own
            .map(|(name, _)| SqlBareRead {
                column: name.clone(),
                tables: Vec::new(),
            })
            .chain(frame.reads)
            .collect();
        frame.scope.resolve_reads(&mut reads);
        up.extend(frame.escaping_qualifiers);
        reads.extend(frame.escaping_reads);
        if let Some(enclosing) = &frame.enclosing {
            up.retain(|qualifier| !enclosing.relations.contains(qualifier));
            enclosing.resolve_reads(&mut reads);
        }
        match self.stack.last_mut() {
            Some(parent) => {
                if frame.enclosing.is_some() {
                    // The derived relation's columns are not visible inside its own query.
                    parent.escaping_qualifiers.extend(up);
                    parent.escaping_reads.extend(reads);
                } else {
                    parent.qualifiers.extend(up);
                    parent.reads.extend(reads);
                }
            }
            None => {
                self.unresolved.extend(up);
                self.reads = reads;
            }
        }
        ControlFlow::Continue(())
    }
}
