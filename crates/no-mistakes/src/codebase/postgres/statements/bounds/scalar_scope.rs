use super::Scope;
use crate::codebase::postgres::catalog::decoded_parts;
use crate::codebase::postgres::statements::{
    SqlBareRead, SqlBoundItem, SqlBoundItemKind, SqlScalarReads,
};
use std::{collections::BTreeSet, rc::Rc};

#[derive(Clone, Default)]
pub(super) struct ScalarScope {
    pub(super) names: BTreeSet<Vec<String>>,
    tables: BTreeSet<(String, Vec<String>)>,
    columns: BTreeSet<String>,
}

impl Scope {
    pub(super) fn with_scalar_scope(&self, items: &[SqlBoundItem]) -> Self {
        let mut scope = self.clone();
        let outer = Rc::make_mut(&mut scope.scalar_scope);
        for item in items {
            if let Some(alias) = &item.alias {
                outer.names.insert(vec![alias.clone()]);
            }
            match &item.kind {
                SqlBoundItemKind::Table(name) => {
                    if !item.alias_explicit {
                        outer.names.insert(decoded_parts(name));
                    }
                    if item.possible_temporary.is_none() {
                        outer
                            .tables
                            .insert((name.clone(), item.column_aliases.clone()));
                    }
                }
                SqlBoundItemKind::Query(query) => {
                    for (index, output) in query.outputs.iter().enumerate() {
                        if let Some(name) = item.column_aliases.get(index).or(output.name.as_ref())
                        {
                            outer.columns.insert(name.clone());
                        }
                    }
                }
                SqlBoundItemKind::Other => {
                    outer.columns.extend(item.column_aliases.iter().cloned());
                }
                _ => {}
            }
        }
        scope
    }

    pub(super) fn scalar_reads(&self, reads: Vec<SqlBareRead>) -> SqlScalarReads {
        SqlScalarReads {
            reads,
            outer_tables: self.scalar_scope.tables.iter().cloned().collect(),
            outer_columns: self.scalar_scope.columns.iter().cloned().collect(),
        }
    }
}
