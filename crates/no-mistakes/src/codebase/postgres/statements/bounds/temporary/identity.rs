//! Catalog-independent candidates retain the database condition until evaluation.
use super::state::{key, Dependency, State};
use crate::codebase::postgres::{decoded_parts, statements::SqlPossibleTemporary};
use crate::codebase::postgres::{SchemaCatalog, SearchPathResolution};
use std::collections::BTreeSet;

pub(super) fn database(name: &str) -> Option<String> {
    match decoded_parts(name).as_slice() {
        [database, schema, _] if schema == "pg_temp" => Some(database.clone()),
        _ => None,
    }
}

impl State {
    pub fn resolves_temporary(&self, name: &str, catalog: Option<&SchemaCatalog>) -> bool {
        let Some(candidate) = self.possible_temporary(name) else {
            return false;
        };
        if candidate.uncertain_lifetime {
            return false;
        }
        let Some(catalog) = catalog else {
            return self.matches_identity(name);
        };
        candidate
            .database_qualifier
            .as_deref()
            .is_none_or(|database| catalog.current_database() == Some(database))
            && matches!(
                catalog.resolve_before_temp(&candidate.earlier_schemas, name),
                SearchPathResolution::Temporary
            )
    }

    pub fn possible_temporary(&self, name: &str) -> Option<SqlPossibleTemporary> {
        let parts = decoded_parts(name);
        let stored = self.databases.get(&key(name));
        let (database_qualifier, mut earlier_schemas) = match parts.as_slice() {
            [_] if self.temp_first => (stored.cloned(), Vec::new()),
            [_] => (stored.cloned(), self.earlier_schemas.clone()?),
            [schema, _] if schema == "pg_temp" => (stored.cloned(), Vec::new()),
            [database, schema, _] if schema == "pg_temp" => {
                if stored.is_some_and(|stored| stored != database) {
                    return None;
                }
                (Some(database.clone()), Vec::new())
            }
            _ => return None,
        };
        let relation = key(name);
        earlier_schemas.retain(|schema| {
            !self
                .removed_shadows
                .contains(&(schema.clone(), relation.clone()))
        });
        self.relations
            .contains_key(&relation)
            .then_some(SqlPossibleTemporary {
                database_qualifier,
                earlier_schemas,
                uncertain_lifetime: self.uncertain_relations.contains(&relation),
            })
    }
    pub fn contains(&self, name: &str) -> bool {
        self.possible_temporary(name).is_some_and(|candidate| {
            candidate.database_qualifier.is_none()
                && candidate.earlier_schemas.is_empty()
                && !candidate.uncertain_lifetime
        })
    }
    pub fn matches_identity(&self, name: &str) -> bool {
        let parts = decoded_parts(name);
        (self.contains(name)
            || self.possible_temporary(name).is_some_and(|candidate| {
                candidate.database_qualifier.is_some()
                    && candidate.earlier_schemas.is_empty()
                    && !candidate.uncertain_lifetime
            }))
            && (parts.len() != 3 || self.databases.get(&key(name)) == parts.first())
    }
    pub fn include_dependencies(&self, name: &str, out: &mut BTreeSet<Dependency>) {
        if let Some(candidate) = self.possible_temporary(name) {
            if !candidate.earlier_schemas.is_empty() {
                // Retain both resolutions until per-catalog evidence settles the source.
                out.insert(Dependency::Physical(decoded_parts(name)));
                out.insert(Dependency::PossibleTemporary(key(name)));
                return;
            }
            out.insert(match candidate.database_qualifier {
                Some(database) => Dependency::ConditionalTemporary(key(name), database),
                None => Dependency::Temporary(key(name)),
            });
        } else {
            out.insert(Dependency::Physical(decoded_parts(name)));
        }
    }
    pub fn insert(
        &mut self,
        name: &str,
        dependencies: BTreeSet<Dependency>,
        inherited: Option<String>,
    ) {
        let key = key(name);
        self.uncertain_relations.remove(&key);
        self.databases.remove(&key);
        if let Some(database) = database(name).or(inherited) {
            self.databases.insert(key.clone(), database);
        }
        self.relations.insert(key, dependencies);
    }
    pub fn dependency_database(&self, dependencies: &BTreeSet<Dependency>) -> Option<String> {
        dependencies.iter().find_map(|dependency| match dependency {
            Dependency::ConditionalTemporary(_, database) => Some(database.clone()),
            _ => None,
        })
    }
}
