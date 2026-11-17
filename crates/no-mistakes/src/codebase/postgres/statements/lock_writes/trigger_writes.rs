use crate::codebase::postgres::dml::writes::names;
use crate::codebase::postgres::{CatalogTriggerWrite, TriggerEvent};
use sqlparser::ast::{FromTable, OnConflictAction, OnInsert, Statement};

pub(super) fn collect(statement: &Statement, out: &mut Vec<CatalogTriggerWrite>) {
    let (relation, event, columns) = match statement {
        Statement::Insert(insert) => (
            names::table_object_name(&insert.table),
            TriggerEvent::Insert,
            Vec::new(),
        ),
        Statement::Update(update) => (
            names::table_factor_name(&update.table.relation),
            TriggerEvent::Update,
            update
                .assignments
                .iter()
                .flat_map(|assignment| names::assignment_column_keys(&assignment.target))
                .collect(),
        ),
        Statement::Delete(delete) => {
            let (FromTable::WithFromKeyword(from) | FromTable::WithoutKeyword(from)) = &delete.from;
            let [table] = from.as_slice() else {
                return;
            };
            (
                names::table_factor_name(&table.relation),
                TriggerEvent::Delete,
                Vec::new(),
            )
        }
        _ => return,
    };
    if let Some(relation) = relation {
        if let Statement::Insert(insert) = statement {
            if let Some(OnInsert::OnConflict(conflict)) = &insert.on {
                if let OnConflictAction::DoUpdate(update) = &conflict.action {
                    out.push(CatalogTriggerWrite {
                        relation: relation.clone(),
                        event: TriggerEvent::Update,
                        columns: update
                            .assignments
                            .iter()
                            .flat_map(|assignment| {
                                names::assignment_column_keys(&assignment.target)
                            })
                            .collect(),
                    });
                }
            }
        }
        out.push(CatalogTriggerWrite {
            relation,
            event,
            columns,
        });
    }
}
