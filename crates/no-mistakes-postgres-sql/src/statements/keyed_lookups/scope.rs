use crate::idents::{ident_key, object_name_identity, relation_suffix_name};
use sqlparser::ast::{Ident, ObjectName, TableAlias};
use std::collections::BTreeMap;

#[derive(Clone)]
struct Binding {
    id: usize,
    qualifier: String,
    columns: Option<BTreeMap<String, Option<usize>>>,
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
        self.add_binding(qualifier, None)
    }

    pub(super) fn add_returning_aliases(&mut self, current_row: Option<&str>) {
        let frame = self.frames.last().expect("RETURNING requires a scope");
        let target_id = frame.first().map(|binding| binding.id);
        for qualifier in ["old", "new"] {
            if self
                .frames
                .iter()
                .flatten()
                .any(|b| b.qualifier == qualifier)
            {
                continue;
            }
            if let Some(id) = target_id.filter(|_| current_row == Some(qualifier)) {
                self.frames
                    .last_mut()
                    .expect("RETURNING requires a scope")
                    .push(Binding {
                        id,
                        qualifier: qualifier.to_owned(),
                        columns: None,
                    });
            } else {
                self.add_named_alias(qualifier.to_owned());
            }
        }
    }

    pub(super) fn add_using_alias(
        &mut self,
        qualifier: String,
        columns: BTreeMap<String, Option<usize>>,
    ) {
        self.add_binding(qualifier, Some(columns));
    }

    fn add_binding(
        &mut self,
        qualifier: String,
        columns: Option<BTreeMap<String, Option<usize>>>,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.frames
            .last_mut()
            .expect("bindings require a query scope")
            .push(Binding {
                id,
                qualifier,
                columns,
            });
        id
    }

    pub(super) fn binding_count(&self) -> usize {
        self.frames
            .last()
            .expect("bindings require a query scope")
            .len()
    }

    pub(super) fn with_local_bindings<T>(
        &mut self,
        start: usize,
        inspect: impl FnOnce(&mut Self) -> T,
    ) -> T {
        if start == 0 {
            return inspect(self);
        }
        let frame = self
            .frames
            .last_mut()
            .expect("bindings require a query scope");
        let visible = frame.split_off(start);
        let mut hidden = std::mem::replace(frame, visible);
        let result = inspect(self);
        let frame = self
            .frames
            .last_mut()
            .expect("bindings require a query scope");
        hidden.append(frame);
        *frame = hidden;
        result
    }

    pub(super) fn truncate_bindings(&mut self, count: usize) {
        self.frames
            .last_mut()
            .expect("bindings require a query scope")
            .truncate(count);
    }

    pub(super) fn refresh_bindings(&mut self, bindings: std::ops::Range<usize>) {
        let frame = self
            .frames
            .last_mut()
            .expect("bindings require a query scope");
        let mut refreshed = BTreeMap::new();
        for binding in &mut frame[bindings] {
            refreshed.insert(binding.id, self.next_id);
            binding.id = self.next_id;
            self.next_id += 1;
        }
        for columns in frame
            .iter_mut()
            .filter_map(|binding| binding.columns.as_mut())
        {
            for id in columns.values_mut().flatten() {
                if let Some(new) = refreshed.get(id) {
                    *id = *new;
                }
            }
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
    pub(super) fn resolve_column(&self, qualifier: &str, column: &str) -> Option<usize> {
        for frame in self.frames.iter().rev() {
            let mut matches = frame.iter().filter(|b| b.qualifier == qualifier);
            if let Some(first) = matches.next() {
                if matches.next().is_some() {
                    return None;
                }
                return match &first.columns {
                    None => Some(first.id),
                    Some(columns) => columns
                        .get(column)
                        .map(|binding| binding.unwrap_or(first.id)),
                };
            }
        }
        None
    }
}
