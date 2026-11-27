use crate::idents::{ident_key, object_name_identity, relation_suffix_name};
use sqlparser::ast::{Ident, ObjectName, TableAlias};

#[derive(Clone)]
struct Binding {
    id: usize,
    qualifier: String,
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
        let qualifier = alias.map_or_else(
            || relation_suffix_name(&object_name_identity(name)),
            ident_key,
        );
        self.add_named_alias(qualifier)
    }

    pub(super) fn add_alias_only(&mut self, alias: Option<&TableAlias>) -> Option<usize> {
        Some(self.add_named_alias(ident_key(&alias?.name)))
    }

    pub(super) fn add_named_alias(&mut self, qualifier: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.frames
            .last_mut()
            .expect("bindings require a query scope")
            .push(Binding { id, qualifier });
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
            let mut matches = frame.iter().filter(|b| b.qualifier == qualifier);
            if let Some(first) = matches.next() {
                return matches.next().is_none().then_some(first.id);
            }
        }
        None
    }
}
