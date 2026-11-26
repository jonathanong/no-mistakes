use crate::codebase::postgres::idents::{ident_key, object_name_identity, relation_suffix_name};
use sqlparser::ast::{Ident, ObjectName, TableAlias};

#[derive(Clone)]
pub(super) struct Binding {
    pub(super) id: usize,
    pub(super) relation: String,
    pub(super) qualifier: String,
    pub(super) has_alias: bool,
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

    pub(super) fn add_table(&mut self, name: &ObjectName, alias: Option<&TableAlias>) -> Binding {
        self.add_table_alias(name, alias.map(|alias| &alias.name))
    }

    pub(super) fn add_table_alias(&mut self, name: &ObjectName, alias: Option<&Ident>) -> Binding {
        let relation = object_name_identity(name);
        let (qualifier, has_alias) = match alias {
            Some(alias) => (ident_key(alias), true),
            None => (relation_suffix_name(&relation), false),
        };
        let binding = Binding {
            id: self.next_id,
            relation,
            qualifier,
            has_alias,
        };
        self.next_id += 1;
        if let Some(frame) = self.frames.last_mut() {
            frame.push(binding.clone());
        }
        binding
    }

    pub(super) fn add_alias_only(&mut self, alias: Option<&TableAlias>) {
        let Some(alias) = alias else { return };
        let qualifier = ident_key(&alias.name);
        let binding = Binding {
            id: self.next_id,
            relation: qualifier.clone(),
            qualifier,
            has_alias: true,
        };
        self.next_id += 1;
        if let Some(frame) = self.frames.last_mut() {
            frame.push(binding);
        }
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

    pub(super) fn single_current_binding(&self) -> Option<usize> {
        let frame = self.frames.last()?;
        (frame.len() == 1).then(|| frame[0].id)
    }
}
