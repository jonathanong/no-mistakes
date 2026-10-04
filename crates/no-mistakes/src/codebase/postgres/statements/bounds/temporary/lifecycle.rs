//! Transaction and DDL transitions for prepared temporary identities.
mod alter_table;
use super::super::items;
use super::{state, TemporaryRelations};
use crate::codebase::postgres::idents::ident_key;
use crate::codebase::postgres::SchemaCatalog;
pub(super) use alter_table::partition_name;
use sqlparser::ast::{ContextModifier, ObjectType, Reset, Set, Statement};

impl TemporaryRelations {
    pub(super) fn lifecycle(&mut self, statement: &Statement, catalog: Option<&SchemaCatalog>) {
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
                if let Some((temp_first, earlier_schemas, search_path)) =
                    self.state.local_path.take()
                {
                    self.state.temp_first = temp_first;
                    self.state.earlier_schemas = earlier_schemas;
                    self.state.search_path = search_path;
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
                self.state.partitioned.clear();
                self.state.on_commit_drop.clear();
                self.state.databases.clear();
                self.state.uncertain_relations.clear();
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
                self.drop_relations(object_type, names, *cascade, catalog)
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
            Statement::AlterTable(table) => self.alter_table(table, catalog),
            Statement::Set(Set::SingleAssignment {
                variable,
                values,
                scope,
                ..
            }) if state::key(&items::sql_name(variable)) == "search_path"
                && (*scope != Some(ContextModifier::Local) || self.transaction.is_some()) =>
            {
                if *scope == Some(ContextModifier::Local) && self.state.local_path.is_none() {
                    self.state.local_path = Some((
                        self.state.temp_first,
                        self.state.earlier_schemas.clone(),
                        self.state.search_path.clone(),
                    ));
                } else if *scope != Some(ContextModifier::Local) {
                    // A session assignment replaces a preceding LOCAL assignment at commit.
                    self.state.local_path = None;
                }
                let path = self.state.record_path(values);
                let pg_temp = path
                    .as_ref()
                    .and_then(|parts| parts.iter().position(|name| name == "pg_temp"));
                let earlier = pg_temp.and_then(|index| {
                    path.as_ref().map(|parts| {
                        let mut earlier = parts[..index].to_vec();
                        // PostgreSQL searches an omitted pg_catalog before even an explicitly
                        // listed pg_temp. A catalog relation can therefore shadow a temp name.
                        if !parts.iter().any(|name| name == "pg_catalog") {
                            earlier.insert(0, "pg_catalog".into());
                        }
                        earlier
                    })
                });
                self.state.temp_first =
                    path.is_some() && earlier.as_ref().is_none_or(Vec::is_empty);
                self.state.earlier_schemas = earlier.filter(|schemas| !schemas.is_empty());
            }
            Statement::Reset(reset)
                if matches!(&reset.reset, Reset::ALL)
                    || matches!(&reset.reset, Reset::ConfigurationParameter(name) if state::key(&items::sql_name(name)) == "search_path") =>
            {
                self.state.temp_first = true;
                self.state.earlier_schemas = None;
                self.state.search_path = None;
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
        catalog: Option<&SchemaCatalog>,
    ) {
        let names: Vec<_> = names.iter().map(items::sql_name).collect();
        if !cascade && self.state.restrict_blocks_drop(&names, catalog) {
            return;
        }
        for name in names {
            // PostgreSQL has no temporary materialized views: this wrong-kind DROP fails.
            if *kind == ObjectType::MaterializedView && self.state.matches_identity(&name) {
                continue;
            }
            self.state.drop(&name, cascade, catalog);
        }
    }
}
