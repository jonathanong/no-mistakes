use crate::codebase::postgres::idents::{ident_key, object_name_identity, relation_suffix_name};
use sqlparser::ast::{Ident, ObjectName, TableAlias};
use std::collections::BTreeMap;

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
    using_columns: Vec<BTreeMap<String, usize>>,
    next_id: usize,
}

impl ScopeStack {
    pub(super) fn push_frame(&mut self) {
        self.frames.push(Vec::new());
        self.using_columns.push(BTreeMap::new());
    }

    pub(super) fn pop_frame(&mut self) {
        self.frames.pop();
        self.using_columns.pop();
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

    pub(super) fn record_using_column(&mut self, column: String, binding: usize) {
        if let Some(map) = self.using_columns.last_mut() {
            map.entry(column).or_insert(binding);
        }
    }

    pub(super) fn snapshot(&self) -> ScopeSnapshot {
        ScopeSnapshot {
            frames: self.frames.clone(),
            using_columns: self.using_columns.clone(),
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct ScopeSnapshot {
    frames: Vec<Vec<Binding>>,
    using_columns: Vec<BTreeMap<String, usize>>,
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

    pub(super) fn resolve_using_column(&self, column: &str) -> Option<usize> {
        if let Some(binding) = self.using_columns.last().and_then(|map| map.get(column)) {
            return Some(*binding);
        }
        let frame = self.frames.last()?;
        (frame.len() == 1).then(|| frame[0].id)
    }
}
