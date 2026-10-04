//! Transaction and DDL transitions for prepared temporary identities.
use super::super::items;
use super::{state, TemporaryRelations};
use crate::codebase::postgres::idents::ident_key;
use sqlparser::ast::{
    AlterTableOperation, ContextModifier, Expr, ObjectName, ObjectNamePart, ObjectType, Partition,
    RenameTableNameKind, Reset, Set, Statement,
};

impl TemporaryRelations {
    pub(super) fn lifecycle(&mut self, statement: &Statement) {
        match statement {
            // PostgreSQL warns on repeated BEGIN without replacing the active transaction.
            Statement::StartTransaction { .. } if self.transaction.is_none() => {
                self.transaction = Some(self.state.clone());
                self.savepoints.clear();
            }
            Statement::Savepoint { name } => {
                self.savepoints.push((ident_key(name), self.state.clone()))
            }
            Statement::ReleaseSavepoint { name } => self.release_savepoint(name),
            Statement::Rollback {
                savepoint: Some(name),
                ..
            } => {
                if let Some(index) = self
                    .savepoints
                    .iter()
                    .rposition(|(key, _)| *key == ident_key(name))
                {
                    self.state = self.savepoints[index].1.clone();
                    self.savepoints.truncate(index + 1);
                }
            }
            Statement::Rollback { chain, .. } => {
                if let Some(state) = self.transaction.take() {
                    self.state = state;
                }
                self.savepoints.clear();
                if *chain {
                    self.transaction = Some(self.state.clone());
                }
            }
            Statement::Commit { chain, .. } => {
                self.state.commit();
                self.transaction = None;
                self.savepoints.clear();
                if let Some(before) = self.state.local_path.take() {
                    self.state.temp_first = before;
                }
                if *chain {
                    self.transaction = Some(self.state.clone());
                }
            }
            Statement::Discard {
                object_type: sqlparser::ast::DiscardObject::TEMP,
            } => {
                self.state.relations.clear();
                self.state.partitions.clear();
                self.state.on_commit_drop.clear();
            }
            Statement::Discard {
                object_type: sqlparser::ast::DiscardObject::ALL,
            } => {
                self.state = super::state::State::default();
                self.transaction = None;
                self.savepoints.clear();
            }
            Statement::Drop {
                object_type,
                names,
                cascade,
                ..
            } if matches!(
                object_type,
                ObjectType::Table | ObjectType::View | ObjectType::MaterializedView
            ) =>
            {
                self.drop_relations(object_type, names, *cascade)
            }
            Statement::Drop {
                object_type: ObjectType::Schema,
                names,
                cascade: true,
                ..
            } => {
                for name in names {
                    self.state.drop_schema(&items::sql_name(name));
                }
            }
            Statement::AlterSchema(schema) => self.alter_schema(schema),
            Statement::AlterTable(table) => self.alter_table(table),
            Statement::Set(Set::SingleAssignment {
                variable,
                values,
                scope,
                ..
            }) if state::key(&items::sql_name(variable)) == "search_path" => {
                if *scope == Some(ContextModifier::Local) && self.state.local_path.is_none() {
                    self.state.local_path = Some(self.state.temp_first);
                } else if *scope != Some(ContextModifier::Local) {
                    // A session assignment replaces a preceding LOCAL assignment at commit.
                    self.state.local_path = None;
                }
                let names: Option<Vec<Vec<String>>> = values
                    .iter()
                    .map(|value| match value {
                        sqlparser::ast::Expr::Identifier(ident) => Some(vec![ident_key(ident)]),
                        sqlparser::ast::Expr::Value(sqlparser::ast::ValueWithSpan {
                            value: sqlparser::ast::Value::SingleQuotedString(raw),
                            ..
                        }) => Some(raw.split(',').map(state::key).collect()),
                        sqlparser::ast::Expr::Value(sqlparser::ast::ValueWithSpan {
                            value: sqlparser::ast::Value::Number(raw, _),
                            ..
                        }) => Some(vec![raw.clone()]),
                        _ => None,
                    })
                    .collect();
                self.state.temp_first = names.is_some_and(|parts| {
                    parts
                        .into_iter()
                        .flatten()
                        .position(|name| name == "pg_temp")
                        .is_none_or(|index| index == 0)
                });
            }
            Statement::Reset(reset)
                if matches!(&reset.reset, Reset::ALL)
                    || matches!(&reset.reset, Reset::ConfigurationParameter(name) if state::key(&items::sql_name(name)) == "search_path") =>
            {
                self.state.temp_first = true;
                self.state.local_path = None;
            }
            _ => {}
        }
    }

    fn release_savepoint(&mut self, name: &sqlparser::ast::Ident) {
        if let Some(index) = self
            .savepoints
            .iter()
            .rposition(|(key, _)| *key == ident_key(name))
        {
            self.savepoints.truncate(index);
        }
    }

    fn alter_schema(&mut self, schema: &sqlparser::ast::AlterSchema) {
        for operation in &schema.operations {
            if let sqlparser::ast::AlterSchemaOperation::Rename { name } = operation {
                self.state
                    .rename_schema(&items::sql_name(&schema.name), &items::sql_name(name));
            }
        }
    }

    fn drop_relations(
        &mut self,
        kind: &ObjectType,
        names: &[sqlparser::ast::ObjectName],
        cascade: bool,
    ) {
        for name in names {
            let name = items::sql_name(name);
            // PostgreSQL has no temporary materialized views: this wrong-kind DROP fails.
            if *kind == ObjectType::MaterializedView && self.state.contains(&name) {
                continue;
            }
            self.state.drop(&name, cascade);
        }
    }
    fn alter_table(&mut self, table: &sqlparser::ast::AlterTable) {
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
