//! ALTER TABLE transitions for temporary partition ownership.
use super::super::super::items;
use super::super::TemporaryRelations;
use sqlparser::ast::{
    AlterTableOperation, Expr, ObjectName, ObjectNamePart, Partition, RenameTableNameKind,
};

impl TemporaryRelations {
    pub(super) fn alter_table(&mut self, table: &sqlparser::ast::AlterTable) {
        for operation in &table.operations {
            match operation {
                AlterTableOperation::RenameTable { table_name } => {
                    let (RenameTableNameKind::To(name) | RenameTableNameKind::As(name)) =
                        table_name;
                    self.state
                        .rename(&items::sql_name(&table.name), &items::sql_name(name));
                }
                AlterTableOperation::AttachPartition { partition } => {
                    if let Some(child) = partition_name(partition) {
                        self.state
                            .attach_partition(&items::sql_name(&table.name), &child);
                    }
                }
                AlterTableOperation::DetachPartition { partition } => {
                    if let Some(child) = partition_name(partition) {
                        self.state
                            .detach_partition(&items::sql_name(&table.name), &child);
                    }
                }
                _ => {}
            }
        }
    }
}

fn partition_name(partition: &Partition) -> Option<String> {
    let Partition::Expr(expression) = partition else {
        return None;
    };
    let parts = match expression {
        Expr::Identifier(ident) => vec![ObjectNamePart::Identifier(ident.clone())],
        Expr::CompoundIdentifier(idents) => idents
            .iter()
            .cloned()
            .map(ObjectNamePart::Identifier)
            .collect(),
        _ => return None,
    };
    Some(items::sql_name(&ObjectName(parts)))
}
