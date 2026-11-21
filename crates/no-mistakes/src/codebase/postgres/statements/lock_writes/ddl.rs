use super::{helpers::generated_name, scope, LockWriteFacts};
use crate::codebase::postgres::idents::object_name_identity;
use crate::codebase::postgres::statements::value::PlaceholderPositions;
use sqlparser::ast::{
    AlterTableOperation, ColumnOption, Expr, ObjectName, ObjectType, Partition, Query, Statement,
    TableConstraint, TableFactor, Visit, Visitor,
};
use std::{collections::BTreeSet, ops::ControlFlow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DdlEvent {
    Lock(usize),
    Read(String),
    Change(String),
    Related(String, String),
}

pub(super) fn collect(
    statement: &Statement,
    positions: PlaceholderPositions<'_>,
    out: &mut LockWriteFacts,
) {
    let identity =
        |name: &ObjectName| (!generated_name(name, positions)).then(|| object_name_identity(name));
    match statement {
        Statement::Lock(lock) => {
            out.ddl_events.extend(
                (out.table_locks.len()..out.table_locks.len() + lock.tables.len())
                    .map(DdlEvent::Lock),
            );
        }
        Statement::AlterTable(alter) => {
            if let Some(name) = identity(&alter.name) {
                for operation in &alter.operations {
                    match operation {
                        AlterTableOperation::AttachPartition { partition }
                        | AlterTableOperation::DetachPartition { partition } => {
                            if let Some(partition) =
                                partition_name(partition).as_ref().and_then(identity)
                            {
                                out.ddl_events.push(DdlEvent::Change(partition));
                            }
                        }
                        AlterTableOperation::AttachPartitionOf { partition, .. }
                        | AlterTableOperation::DetachPartitionOf { partition, .. } => {
                            if let Some(partition) = identity(partition) {
                                out.ddl_events.push(DdlEvent::Change(partition));
                            }
                        }
                        _ => {}
                    }
                }
                out.ddl_events.push(DdlEvent::Change(name));
            }
        }
        Statement::Drop {
            object_type: ObjectType::Table,
            names,
            ..
        } => {
            out.ddl_events
                .extend(names.iter().filter_map(identity).map(DdlEvent::Change));
        }
        Statement::CreateTable(table) => {
            if let Some(name) = identity(&table.name) {
                let parents = table
                    .inherits
                    .iter()
                    .flatten()
                    .chain(table.partition_of.iter());
                let foreign = table
                    .constraints
                    .iter()
                    .filter_map(|constraint| {
                        if let TableConstraint::ForeignKey(fk) = constraint {
                            Some(&fk.foreign_table)
                        } else {
                            None
                        }
                    })
                    .chain(
                        table
                            .columns
                            .iter()
                            .flat_map(|column| &column.options)
                            .filter_map(|option| {
                                if let ColumnOption::ForeignKey(fk) = &option.option {
                                    Some(&fk.foreign_table)
                                } else {
                                    None
                                }
                            }),
                    );
                for target in parents.chain(foreign).filter_map(identity) {
                    out.ddl_events.push(DdlEvent::Related(name.clone(), target));
                }
            }
        }
        _ => {}
    }
    let _ = statement.visit(&mut Reads {
        events: &mut out.ddl_events,
        scopes: Vec::new(),
        mutations: Vec::new(),
        positions,
    });
}
struct Reads<'a, 'p> {
    events: &'a mut Vec<DdlEvent>,
    scopes: Vec<BTreeSet<String>>,
    positions: PlaceholderPositions<'p>,
    mutations: Vec<usize>,
}
impl Visitor for Reads<'_, '_> {
    type Break = ();
    fn pre_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        if matches!(
            statement,
            Statement::Update(_) | Statement::Delete(_) | Statement::Insert(_)
        ) {
            self.mutations.push(self.scopes.len());
        }
        ControlFlow::Continue(())
    }
    fn post_visit_statement(&mut self, statement: &Statement) -> ControlFlow<()> {
        if matches!(
            statement,
            Statement::Update(_) | Statement::Delete(_) | Statement::Insert(_)
        ) {
            self.mutations.pop();
        }
        ControlFlow::Continue(())
    }
    fn pre_visit_query(&mut self, query: &Query) -> ControlFlow<()> {
        self.scopes.push(scope::cte_names(query));
        ControlFlow::Continue(())
    }
    fn post_visit_query(&mut self, _: &Query) -> ControlFlow<()> {
        self.scopes.pop();
        ControlFlow::Continue(())
    }
    fn pre_visit_table_factor(&mut self, table: &TableFactor) -> ControlFlow<()> {
        if !self.scopes.is_empty()
            && self.scopes.iter().all(BTreeSet::is_empty)
            && self.mutations.last() != Some(&self.scopes.len())
        {
            if let TableFactor::Table {
                name, args: None, ..
            } = table
            {
                let relation = object_name_identity(name);
                if !generated_name(name, self.positions)
                    && !self.scopes.iter().any(|names| names.contains(&relation))
                {
                    self.events.push(DdlEvent::Read(relation));
                }
            }
        }
        ControlFlow::Continue(())
    }
}

fn partition_name(partition: &Partition) -> Option<ObjectName> {
    let Partition::Expr(expression) = partition else {
        return None;
    };
    match expression {
        Expr::Identifier(ident) => Some(ObjectName::from(vec![ident.clone()])),
        Expr::CompoundIdentifier(idents) => Some(ObjectName::from(idents.clone())),
        _ => None,
    }
}
