mod columns;
mod scalar_arrays;
mod sources;
use super::super::super::value::is_placeholder_ident;
use super::super::items::sql_name;
use crate::codebase::postgres::idents::{ident_key, object_name_ident};
use crate::codebase::postgres::statements::SqlBareRead;
pub(in super::super) use columns::projection_columns;
use sqlparser::ast::{
    Expr, GroupByExpr, ObjectName, OrderByKind, Query, SetExpr, TableFactor, Visit, Visitor,
};
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
    outer: &BTreeSet<Vec<String>>,
    ctes: &BTreeMap<String, Option<BTreeSet<String>>>,
) -> Reads {
    let mut scan = Scan {
        ctes: ctes.clone(),
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
    /// A nested WITH clause must not change the relation names visible to its parent.
    previous_ctes: BTreeMap<String, Option<BTreeSet<String>>>,
    /// Identifier components preserve the distinction between a quoted dot and a separator.
    relations: BTreeSet<Vec<String>>,
    /// Bare relation names also identify whole-row references.
    whole_rows: BTreeSet<String>,
    /// The base tables of the level, as SQL names.
    tables: Vec<String>,
    /// A relation that is not a base table: a derived table, a function, a CTE.
    foreign: bool,
    /// Known projected columns of derived/CTE/function sources in this level.
    columns: BTreeSet<String>,
    qualifiers: Vec<Vec<String>>,
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
    ctes: BTreeMap<String, Option<BTreeSet<String>>>,
    /// Qualifiers and bare reads that no level of the query resolved.
    unresolved: Vec<Vec<String>>,
    reads: Vec<SqlBareRead>,
}

impl Scan {
    fn is_cte(&self, name: &ObjectName) -> bool {
        name.0.len() == 1
            && object_name_ident(name)
                .is_some_and(|ident| self.ctes.contains_key(&ident_key(ident)))
    }
}

impl Visitor for Scan {
    type Break = ();

    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        let mut frame = Frame {
            previous_ctes: self.ctes.clone(),
            ..Frame::default()
        };
        for name in output_names(query) {
            *frame.labels.entry(name).or_default() += 1;
        }
        if let Some(with) = &query.with {
            for cte in &with.cte_tables {
                let columns = if cte.alias.columns.is_empty() {
                    projection_columns(&cte.query)
                } else {
                    Some(
                        cte.alias
                            .columns
                            .iter()
                            .map(|column| ident_key(&column.name))
                            .collect(),
                    )
                };
                self.ctes.insert(ident_key(&cte.alias.name), columns);
            }
        }
        self.stack.push(frame);
        ControlFlow::Continue(())
    }

    /// The relations of a level are complete only after it has been visited (the projection comes
    /// before FROM), so its references are resolved here and what remains moves up a level.
    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        let frame = self.stack.pop().unwrap_or_default();
        self.ctes = frame.previous_ctes;
        let up: Vec<Vec<String>> = frame
            .qualifiers
            .into_iter()
            .filter(|qualifier| !frame.relations.contains(qualifier))
            .collect();
        // A bare name that is a relation's own is a whole-row reference, not a column.
        let own = frame.bare.iter().filter(|(name, count)| {
            **count > frame.labels.get(*name).copied().unwrap_or(0)
                && !frame.whole_rows.contains(*name)
        });
        let mut reads: Vec<SqlBareRead> = own
            .map(|(name, _)| SqlBareRead {
                column: name.clone(),
                tables: Vec::new(),
            })
            .chain(frame.reads)
            .collect();
        if frame.foreign {
            reads.clear();
        } else {
            reads.retain(|read| !frame.columns.contains(&read.column));
        }
        for read in &mut reads {
            read.tables.extend(frame.tables.iter().cloned());
            read.tables.sort();
            read.tables.dedup();
        }
        match self.stack.last_mut() {
            Some(parent) => {
                parent.qualifiers.extend(up);
                parent.reads.extend(reads);
            }
            None => {
                self.unresolved.extend(up);
                self.reads = reads;
            }
        }
        ControlFlow::Continue(())
    }

    fn pre_visit_table_factor(&mut self, factor: &TableFactor) -> ControlFlow<()> {
        self.add_factor(factor);
        ControlFlow::Continue(())
    }

    fn pre_visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        let Some(frame) = self.stack.last_mut() else {
            return ControlFlow::Continue(());
        };
        match expr {
            Expr::CompoundIdentifier(parts) if parts.len() >= 2 => {
                frame
                    .qualifiers
                    .push(parts[..parts.len() - 1].iter().map(ident_key).collect());
            }
            Expr::Identifier(ident) if !is_placeholder_ident(&ident.value) => {
                *frame.bare.entry(ident_key(ident)).or_default() += 1;
            }
            _ => {}
        }
        ControlFlow::Continue(())
    }
}

/// The names written alone as an `ORDER BY` or `GROUP BY` item: PostgreSQL reads each as an
/// output column's name before it reads it as a column of a relation.
fn output_names(query: &Query) -> Vec<String> {
    let mut items: Vec<&Expr> = Vec::new();
    if let Some(order) = &query.order_by {
        if let OrderByKind::Expressions(expressions) = &order.kind {
            items.extend(expressions.iter().map(|expression| &expression.expr));
        }
    }
    if let SetExpr::Select(select) = &*query.body {
        if let GroupByExpr::Expressions(expressions, _) = &select.group_by {
            items.extend(expressions);
        }
    }
    items
        .into_iter()
        .filter_map(|item| match item {
            Expr::Identifier(ident) if !is_placeholder_ident(&ident.value) => {
                Some(ident_key(ident))
            }
            _ => None,
        })
        .collect()
}
