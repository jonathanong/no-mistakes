use super::{EdgeIndex, EdgeKind, ExportKey, ExportLookup, ExportProof, FxHashMap};
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
                    Some(ExportProof::Playwright(offset)) => {
                        Binding::Playwright(offset.map(|offset| (key.file.as_path(), offset)))
                    }
                    Some(ExportProof::Alias(offset)) => match upstream {
                        Binding::Playwright(identity) => Binding::Playwright(
                            offset
                                .map(|offset| (key.file.as_path(), offset))
                                .or(identity),
                        ),
                        _ => Binding::Unproven,
                    },
                    Some(ExportProof::Stars) => upstream,
                    Some(ExportProof::Unknown) | None => Binding::Unproven,
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
}
