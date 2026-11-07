use super::*;
use crate::codebase::dependencies::extract::{
    CallableId, CatalogReadKind, LiteralCatalog, ValueBinding, ValueCall, ValueExpression,
    ValueFlowFacts,
};
use crate::fx::{FxHashMap, FxHashSet};

mod callers;
mod helpers;
mod index;
mod namespace_escapes;
mod pipelines;

type Helper<'a> = (&'a Path, CallableId);
type Call<'a> = (&'a Path, u32, u32);
type Source<'a> = (&'a Path, usize, &'a str);

pub(super) struct CatalogProof<'a, 'b> {
    index: index::Index<'a>,
    bindings: &'a Bindings<'b>,
    bounded_calls: FxHashMap<Call<'a>, &'a [Vec<String>]>,
    safe_sources: FxHashSet<Source<'a>>,
}

impl<'a, 'b> CatalogProof<'a, 'b> {
    pub(super) fn new(
        graph: &'a DepGraph,
        facts: &'a CheckFactMap,
        bindings: &'a Bindings<'b>,
    ) -> Self {
        let index = index::Index::new(graph, facts);
        let mut result = Self {
            index,
            bindings,
            bounded_calls: FxHashMap::default(),
            safe_sources: FxHashSet::default(),
        };
        result.prepare(graph, facts);
        result
    }

    fn prepare(&mut self, graph: &DepGraph, facts: &CheckFactMap) {
        let escaped = helpers::escaped(&self.index, graph, facts, self.bindings);
        self.index.bad_helpers.extend(escaped);
        let escaped = namespace_escapes::collect(&self.index, graph);
        self.index.bad_helpers.extend(escaped);
        for (source, catalog) in &self.index.sources {
            let mut safe = self.index.source_safe(*source, catalog);
            let mut calls = Vec::new();
            for helper in self.index.source_helpers.get(source).into_iter().flatten() {
                safe &= helpers::safe(&self.index, *helper, *source);
                calls.extend(
                    self.index
                        .incoming
                        .get(helper)
                        .into_iter()
                        .flatten()
                        .copied(),
                );
            }
            if safe {
                self.safe_sources.insert(*source);
                for call in calls {
                    self.bounded_calls.insert(call, &catalog.fields);
                }
            }
        }
    }

    pub(super) fn proven(&self, call: &SqlCall<'_>, provenance: &Provenance) -> bool {
        let Provenance::Reference {
            name,
            scope,
            offset,
            members,
        } = provenance
        else {
            return false;
        };
        let Some(binding) = self.bindings.value(&call.file, *scope, name, *offset) else {
            return false;
        };
        if !binding.immutable || binding.invalidated_at.is_some() {
            return false;
        }
        self.scalar(
            &call.file,
            &binding.value,
            members,
            &mut FxHashSet::default(),
        ) || members.is_empty() && self.pipeline(&call.file, &binding.value)
    }

    fn scalar(
        &self,
        file: &Path,
        value: &ValueExpression,
        members: &[String],
        seen: &mut FxHashSet<(usize, String, u32)>,
    ) -> bool {
        match value {
            ValueExpression::Member(base, member) => {
                let mut path = vec![member.clone()];
                path.extend_from_slice(members);
                self.scalar(file, base, &path, seen)
            }
            ValueExpression::Call(start, end) => self
                .bounded_calls
                .get(&(file, *start, *end))
                .is_some_and(|fields| fields.iter().any(|field| field == members)),
            ValueExpression::Named {
                name,
                scope: Some(scope),
                offset,
                ..
            } => {
                if !seen.insert((*scope, name.clone(), *offset)) {
                    return false;
                }
                self.bindings
                    .value(file, *scope, name, *offset)
                    .is_some_and(|binding| {
                        binding.immutable
                            && binding.invalidated_at.is_none()
                            && self.scalar(file, &binding.value, members, seen)
                    })
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod coverage_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
