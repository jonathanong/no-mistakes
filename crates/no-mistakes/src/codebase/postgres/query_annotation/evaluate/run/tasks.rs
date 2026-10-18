use super::super::File;
use super::{
    entrypoints::Functions,
    reachability::{export_reachable_globals, ExportReachability},
};
use crate::codebase::postgres::query_annotation::{Expr, Function};
use std::{collections::btree_map, slice, sync::Arc};

pub(super) enum Task<'a> {
    Call(&'a Expr),
    Function(Arc<Function>),
}

// The caller keeps at most two yielded tasks; traversal stores borrowed DFS
// frames rather than a vector of every function or every container child.
pub(super) struct Tasks<'a> {
    file: &'a File<'a>,
    unmodeled: slice::Iter<'a, Expr>,
    globals: btree_map::Iter<'a, String, Expr>,
    functions: Functions<'a>,
    functions_started: bool,
    reachable: Option<ExportReachability<'a>>,
}
impl<'a> Tasks<'a> {
    pub(super) fn new(file: &'a File<'a>) -> Self {
        Self {
            file,
            unmodeled: file.facts.unmodeled_calls.iter(),
            globals: file.facts.globals.iter(),
            functions: Functions::new(&file.facts.globals),
            functions_started: false,
            reachable: None,
        }
    }
}
impl<'a> Iterator for Tasks<'a> {
    type Item = Task<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.functions_started {
            for call in self.unmodeled.by_ref() {
                if self.file.facts.precise_records
                    && matches!(call, Expr::Call { start, end, .. }
                    if self.file.facts.branch_calls.contains(&(*start, *end)))
                {
                    continue;
                }
                if matches!(call, Expr::Call { start, end, .. }
                    if !self.file.executors.contains(&(*start, *end)))
                {
                    return Some(Task::Call(call));
                }
            }
            self.functions_started = true;
        }
        if !self.file.facts.precise_records {
            for (_, value) in self.globals.by_ref() {
                if let Expr::Function(function) = value {
                    return Some(Task::Function(Arc::clone(function)));
                }
            }
            return None;
        }
        // Initialize only when lookahead actually reaches the function phase;
        // two eligible unmodeled calls need no early export traversal.
        let file = self.file;
        let reachable = self.reachable.get_or_insert_with(|| {
            export_reachable_globals(
                &file.facts.globals,
                file.module
                    .ts
                    .exported_bindings
                    .iter()
                    .filter(|binding| binding.specifier.is_none())
                    .map(|binding| binding.local.as_str()),
            )
        });
        loop {
            if let Some(function) = self.functions.next() {
                return Some(Task::Function(function));
            }
            let (name, value) = self.globals.next()?;
            if reachable.contains(name) {
                self.functions.start(value);
            }
        }
    }
}

#[cfg(test)]
#[path = "tasks_tests.rs"]
mod tests;
