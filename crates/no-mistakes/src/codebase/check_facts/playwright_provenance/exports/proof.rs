use super::{
    EdgeIndex, EdgeKind, ExportKey, ExportLookup, ExportProof, FxHashMap, FxHashSet, WrapperProof,
};
use std::collections::hash_map::Entry;
use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Binding<'a> {
    Missing,
    Unproven,
    Playwright(Option<(&'a Path, u32)>),
}

impl<'a> Binding<'a> {
    fn join(self, other: Self) -> Self {
        match (self, other) {
            (Self::Missing, result) | (result, Self::Missing) => result,
            (a, b) if a == b => a,
            _ => Self::Unproven,
        }
    }
}

impl ExportLookup<'_> {
    pub(in crate::codebase::check_facts::playwright_provenance) fn proves_playwright(
        &self,
        relationships: &EdgeIndex<ExportKey, EdgeKind>,
        root: &ExportKey,
    ) -> bool {
        let mut bindings = FxHashMap::default();
        let mut pending = vec![(root, false)];
        while let Some((key, finish)) = pending.pop() {
            let targets = || {
                relationships
                    .forward()
                    .get(key)
                    .into_iter()
                    .flat_map(|neighbors| neighbors.iter().map(|(target, _)| target))
            };
            if finish {
                let upstream = targets()
                    .map(|target| bindings.get(target).copied().unwrap_or(Binding::Unproven))
                    .fold(Binding::Missing, Binding::join);
                let proven = match self.proof.get(key) {
                    Some(ExportProof::Playwright(offset, deps)) if self.all_wrappers_ok(deps) => {
                        Binding::Playwright(offset.map(|offset| (key.file.as_path(), offset)))
                    }
                    Some(ExportProof::Alias(offset, deps)) if self.all_wrappers_ok(deps) => {
                        match upstream {
                            Binding::Playwright(identity) => Binding::Playwright(
                                offset
                                    .map(|offset| (key.file.as_path(), offset))
                                    .or(identity),
                            ),
                            _ => Binding::Unproven,
                        }
                    }
                    Some(ExportProof::Stars) => upstream,
                    _ => Binding::Unproven,
                };
                bindings.insert(key, proven);
            } else if let Entry::Vacant(binding) = bindings.entry(key) {
                binding.insert(Binding::Unproven);
                pending.push((key, true));
                pending.extend(targets().map(|target| (target, false)));
            }
        }
        matches!(bindings.get(root), Some(Binding::Playwright(_)))
    }

    pub(in crate::codebase::check_facts::playwright_provenance) fn proves_wrapper(
        &self,
        key: &ExportKey,
        expected_param: usize,
    ) -> bool {
        let mut visiting = FxHashSet::default();
        self.proves_wrapper_inner(key, expected_param, &mut visiting)
    }

    fn all_wrappers_ok(&self, deps: &[(ExportKey, usize)]) -> bool {
        deps.iter()
            .all(|(target, arg_index)| self.proves_wrapper(target, *arg_index))
    }

    fn proves_wrapper_inner(
        &self,
        key: &ExportKey,
        expected_param: usize,
        visiting: &mut FxHashSet<(ExportKey, usize)>,
    ) -> bool {
        if !visiting.insert((key.clone(), expected_param)) {
            return false;
        }
        let ok = match self.wrappers.get(key) {
            Some(WrapperProof::Direct { param_index, deps }) => {
                *param_index == expected_param
                    && deps.iter().all(|(target, arg_index)| {
                        self.proves_wrapper_inner(target, *arg_index, visiting)
                    })
            }
            Some(WrapperProof::Alias(target)) => {
                self.proves_wrapper_inner(target, expected_param, visiting)
            }
            Some(WrapperProof::Stars(targets)) => {
                let mut matched = 0;
                for target in targets {
                    if self.proves_wrapper_inner(target, expected_param, visiting) {
                        matched += 1;
                    }
                }
                matched == 1
            }
            Some(WrapperProof::Unknown) | None => false,
        };
        visiting.remove(&(key.clone(), expected_param));
        ok
    }

    pub(in crate::codebase::check_facts::playwright_provenance) fn reaches_uncertain(
        &self,
        relationships: &EdgeIndex<ExportKey, EdgeKind>,
        root: &ExportKey,
    ) -> bool {
        self.reachable_exports(relationships, root)
            .iter()
            .any(|key| self.uncertain.contains(key))
    }

    pub(in crate::codebase::check_facts::playwright_provenance) fn reachable_exports(
        &self,
        relationships: &EdgeIndex<ExportKey, EdgeKind>,
        root: &ExportKey,
    ) -> Vec<ExportKey> {
        let mut seen = FxHashSet::default();
        let mut result = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(key) = stack.pop() {
            if !seen.insert(key.clone()) {
                continue;
            }
            result.push(key.clone());
            if let Some(neighbors) = relationships.forward().get(&key) {
                for (target, _) in neighbors {
                    stack.push(target.clone());
                }
            }
        }
        result
    }
}
