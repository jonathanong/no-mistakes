use super::*;
use crate::codebase::dependencies::extract::{ValueCall, ValueFlowFacts};
use crate::fx::{FxHashMap, FxHashSet};
use std::path::PathBuf;

#[path = "fragment_builder_bindings.rs"]
mod bindings;
#[path = "fragment_builder_helpers.rs"]
mod helpers;
#[path = "fragment_builder_imports.rs"]
mod imports;
#[path = "fragment_builder_lifecycle.rs"]
mod lifecycle;

#[derive(Clone, Hash, PartialEq, Eq)]
enum Key {
    Binding(PathBuf, usize, String, u32, bool, bool),
    Call(PathBuf, u32, u32, bool),
    Imports(PathBuf, String),
}

struct File<'a> {
    flow: &'a ValueFlowFacts,
    calls: FxHashMap<(u32, u32), &'a ValueCall>,
}

pub(super) struct Proof<'a, 'facts> {
    graph: &'a DepGraph,
    opts: &'a CompiledOptions,
    facts: &'a CheckFactMap,
    bindings: &'a Bindings<'facts>,
    files: FxHashMap<PathBuf, File<'a>>,
    importers: FxHashMap<PathBuf, Option<Vec<PathBuf>>>,
    active: FxHashSet<Key>,
    done: FxHashMap<Key, bool>,
}

impl<'a, 'facts> Proof<'a, 'facts> {
    pub(super) fn new(
        graph: &'a DepGraph,
        opts: &'a CompiledOptions,
        facts: &'a CheckFactMap,
        bindings: &'a Bindings<'facts>,
    ) -> Self {
        Self {
            graph,
            opts,
            facts,
            bindings,
            files: FxHashMap::default(),
            importers: FxHashMap::default(),
            active: FxHashSet::default(),
            done: FxHashMap::default(),
        }
    }

    fn file(&mut self, file: &Path) -> Option<&File<'a>> {
        if !self.files.contains_key(file) {
            let ts = self.facts.get_ts_facts(file)?;
            if ts.parse_error.is_some()
                || ts.operational_error.is_some()
                || ts.value_flow.incomplete_reason.is_some()
            {
                return None;
            }
            self.files.insert(
                file.to_path_buf(),
                File {
                    flow: &ts.value_flow,
                    calls: ts
                        .value_flow
                        .calls
                        .iter()
                        .map(|call| ((call.offset, call.end), call))
                        .collect(),
                },
            );
        }
        self.files.get(file)
    }

    fn expression(
        &mut self,
        file: &Path,
        expression: &ValueExpression,
        default_only: bool,
    ) -> bool {
        match expression {
            ValueExpression::Named {
                name,
                scope,
                offset,
                ..
            } => self.binding(file, scope.unwrap_or(0), name, *offset, default_only),
            ValueExpression::Call(start, end) => {
                let Some(call) = self
                    .file(file)
                    .and_then(|data| data.calls.get(&(*start, *end)))
                    .copied()
                else {
                    return false;
                };
                self.memoized(
                    Key::Call(file.to_path_buf(), *start, *end, default_only),
                    |proof| {
                        if proof.files[file]
                            .flow
                            .tagged_templates
                            .contains_key(&(*start, *end))
                        {
                            let tags = if default_only {
                                &[][..]
                            } else {
                                &proof.opts.embedded.trusted_sql_tags
                            };
                            return super::super::super::postgres_sql_cross_file::is_text_tag(
                                proof.bindings,
                                file,
                                call,
                                tags,
                            );
                        }
                        if let ValueExpression::Member(base, method) = &call.callee {
                            return method == "append"
                                && proof.expression(file, base, true)
                                && proof.append_argument(file, call);
                        }
                        helpers::returned_builder(proof, file, call)
                    },
                )
            }
            _ => false,
        }
    }

    fn append_argument(&mut self, file: &Path, call: &ValueCall) -> bool {
        !call.bound
            && call.spread_at.is_none()
            && call.arguments.len() == 1
            && self.expression(file, &call.arguments[0], true)
    }

    fn memoized(&mut self, key: Key, evaluate: impl FnOnce(&mut Self) -> bool) -> bool {
        if let Some(result) = self.done.get(&key) {
            return *result;
        }
        if !self.active.insert(key.clone()) {
            return false;
        }
        let result = evaluate(self);
        self.active.remove(&key);
        self.done.insert(key, result);
        result
    }
}

#[cfg(test)]
#[path = "fragment_builder_boundary_tests.rs"]
mod boundary_tests;
