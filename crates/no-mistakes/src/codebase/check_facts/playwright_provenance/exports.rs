use super::*;
use crate::codebase::dependencies::{extract::ImportedBindingKind, graph::EdgeKind};
use crate::codebase::rules::test_reliability::playwright_provenance::WrapperCall;
use crate::codebase::ts_resolver::ImportResolution;
use crate::edge_index::{CanonicalEdge, EdgeIndex};
use std::sync::Arc;

mod proof;
mod uncertain;
mod wrappers;
use wrappers::WrapperProof;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ExportKey {
    pub(super) file: PathBuf,
    pub(super) name: Arc<str>,
}

enum Export {
    Playwright(Option<u32>, Vec<(ExportKey, usize)>),
    Local(ExportKey, Option<u32>, Vec<(ExportKey, usize)>),
    Stars(Vec<ExportKey>),
    Unknown,
}

#[derive(Clone)]
enum ExportProof {
    Playwright(Option<u32>, Vec<(ExportKey, usize)>),
    Alias(Option<u32>, Vec<(ExportKey, usize)>),
    Stars,
    Unknown,
}

pub(super) struct ExportLookup<'a> {
    resolver: &'a dyn ImportResolution,
    visible: &'a PathSet,
    proof: FxHashMap<ExportKey, ExportProof>,
    uncertain: FxHashSet<ExportKey>,
    uncertain_deps: FxHashMap<ExportKey, Vec<ExportKey>>,
    wrappers: FxHashMap<ExportKey, WrapperProof>,
    edges: Vec<CanonicalEdge<ExportKey, EdgeKind>>,
}

impl<'a> ExportLookup<'a> {
    pub(super) fn new(resolver: &'a dyn ImportResolution, visible: &'a PathSet) -> Self {
        Self {
            resolver,
            visible,
            proof: FxHashMap::default(),
            uncertain: FxHashSet::default(),
            uncertain_deps: FxHashMap::default(),
            wrappers: FxHashMap::default(),
            edges: Vec::new(),
        }
    }

    pub(super) fn contains(&self, key: &ExportKey) -> bool {
        self.proof.contains_key(key)
    }

    pub(super) fn add(
        &mut self,
        next: &mut Vec<ExportKey>,
        key: ExportKey,
        facts: Option<&CheckFileFacts>,
    ) {
        let (proof, dependencies) = match self.export(facts, &key) {
            Export::Playwright(derived_at, deps) => {
                next.extend(deps.iter().map(|(target, _)| target.clone()));
                (ExportProof::Playwright(derived_at, deps), Vec::new())
            }
            Export::Local(target, derived_at, deps) => {
                next.extend(deps.iter().map(|(dep, _)| dep.clone()));
                (ExportProof::Alias(derived_at, deps), vec![target])
            }
            Export::Stars(targets) => (ExportProof::Stars, targets),
            Export::Unknown => (ExportProof::Unknown, Vec::new()),
        };
        let wrapper_proof = self.wrapper_export(facts, &key);
        match &wrapper_proof {
            WrapperProof::Direct { deps, .. } => {
                next.extend(deps.iter().map(|(dep, _)| dep.clone()));
            }
            WrapperProof::Alias(target) => next.push(target.clone()),
            WrapperProof::Stars(targets) => next.extend(targets.iter().cloned()),
            WrapperProof::Unknown => {}
        }
        next.extend(dependencies.iter().cloned());
        self.edges.extend(
            dependencies
                .into_iter()
                .map(|target| CanonicalEdge::new(key.clone(), target, EdgeKind::SymbolReexport)),
        );
        self.record_uncertain(next, &key, facts);
        self.proof.insert(key.clone(), proof);
        self.wrappers.insert(key, wrapper_proof);
    }

    pub(super) fn local(&self, from: &Path, import: &ImportRef) -> Option<ExportKey> {
        let file = self.resolver.resolve(&import.specifier, from)?;
        (self.visible.contains(&file) && is_indexable(&file)).then(|| ExportKey {
            file,
            name: Arc::clone(&import.name),
        })
    }

    fn imported(
        &self,
        from: &Path,
        import: &ImportRef,
        derived_at: Option<u32>,
        deps: Vec<(ExportKey, usize)>,
    ) -> Export {
        if import.is_playwright_test() {
            return Export::Playwright(derived_at, deps);
        }
        self.local(from, import).map_or(Export::Unknown, |target| {
            Export::Local(target, derived_at, deps)
        })
    }

    fn export(&self, facts: Option<&CheckFileFacts>, key: &ExportKey) -> Export {
        let Some(facts) = facts.filter(|file| file.parse_error.is_none() && file.parsed) else {
            return Export::Unknown;
        };
        let ts = &facts.ts;
        if let Some(binding) = ts
            .exported_bindings
            .iter()
            .find(|binding| binding.exported == key.name.as_ref())
        {
            if let Some(specifier) = &binding.specifier {
                return self.imported(
                    &key.file,
                    &ImportRef::new(specifier, &binding.local),
                    None,
                    Vec::new(),
                );
            }
            if let Some(init) = facts
                .fixture_initializers
                .as_ref()
                .and_then(|fixtures| fixtures.values.get(&binding.local))
            {
                let Some(deps) = self.resolve_wrappers(&key.file, &init.wrappers) else {
                    return Export::Unknown;
                };
                return self.imported(&key.file, &init.import, init.derived_at, deps);
            }
            if let Some(import) = ts.imported_bindings.iter().find(|import| {
                !import.is_type_only
                    && import.kind != ImportedBindingKind::Namespace
                    && import.local == binding.local
            }) {
                return self.imported(
                    &key.file,
                    &ImportRef::new(&import.specifier, &import.imported),
                    None,
                    Vec::new(),
                );
            }
            return Export::Unknown;
        }
        if key.name.as_ref() == "default" {
            return Export::Stars(Vec::new());
        }
        let mut stars = Vec::new();
        for specifier in &ts.star_reexport_specifiers {
            let reference = ImportRef::new(specifier, &key.name);
            if reference.is_playwright_test() {
                return if ts.star_reexport_specifiers.len() == 1 {
                    Export::Playwright(None, Vec::new())
                } else {
                    Export::Unknown
                };
            }
            let Some(target) = self.local(&key.file, &reference) else {
                return Export::Unknown;
            };
            stars.push(target);
        }
        Export::Stars(stars)
    }

    pub(super) fn finalize(&mut self) -> EdgeIndex<ExportKey, EdgeKind> {
        EdgeIndex::from_edges(std::mem::take(&mut self.edges))
    }
}
