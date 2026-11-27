use crate::idents::{ident_key, object_name_identity, relation_suffix_name};
use sqlparser::ast::{Ident, ObjectName, TableAlias};

#[derive(Clone)]
struct Binding {
    id: usize,
    relation: String,
    qualifier: String,
    has_alias: bool,
}

#[derive(Default)]
pub(super) struct ScopeStack {
    frames: Vec<Vec<Binding>>,
    next_id: usize,
}

impl ScopeStack {
    pub(super) fn push_frame(&mut self) {
        self.frames.push(Vec::new());
    }

    pub(super) fn pop_frame(&mut self) {
        self.frames.pop();
    }

    pub(super) fn add_table(&mut self, name: &ObjectName, alias: Option<&TableAlias>) -> usize {
        self.add_table_alias(name, alias.map(|alias| &alias.name))
    }

    pub(super) fn add_table_alias(&mut self, name: &ObjectName, alias: Option<&Ident>) -> usize {
        let relation = object_name_identity(name);
        let (qualifier, has_alias) = match alias {
            Some(alias) => (ident_key(alias), true),
            None => (relation_suffix_name(&relation), false),
        };
        self.add_binding(relation, qualifier, has_alias)
    }

    pub(super) fn add_alias_only(&mut self, alias: Option<&TableAlias>) -> Option<usize> {
        let qualifier = ident_key(&alias?.name);
        Some(self.add_binding(qualifier.clone(), qualifier, true))
    }

    fn add_binding(&mut self, relation: String, qualifier: String, has_alias: bool) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.frames
            .last_mut()
            .expect("bindings require a query scope")
            .push(Binding {
                id,
                relation,
                qualifier,
                has_alias,
            });
        id
    }

    pub(super) fn snapshot(&self) -> ScopeSnapshot {
        ScopeSnapshot {
            frames: self.frames.clone(),
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct ScopeSnapshot {
    frames: Vec<Vec<Binding>>,
}

impl ScopeSnapshot {
    pub(super) fn resolve_qualifier(&self, qualifier: &str) -> Option<usize> {
        for frame in self.frames.iter().rev() {
            let mut matches = frame
                .iter()
                .filter(|b| b.qualifier == qualifier || (!b.has_alias && b.relation == qualifier));
            if let Some(first) = matches.next() {
                return matches.next().is_none().then_some(first.id);
            }
        }
        None
    }
}
