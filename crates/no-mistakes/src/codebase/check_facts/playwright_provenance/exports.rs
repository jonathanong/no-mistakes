use super::*;
use crate::codebase::dependencies::{extract::ImportedBindingKind, graph::EdgeKind};
use crate::codebase::ts_resolver::ImportResolution;
use crate::edge_index::{CanonicalEdge, EdgeIndex};
use std::sync::Arc;

mod proof;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct ExportKey {
    pub(super) file: PathBuf,
    name: Arc<str>,
}

enum Export {
    Playwright(Option<u32>),
    Local(ExportKey, Option<u32>),
    Stars(Vec<ExportKey>),
    Unknown,
}

#[derive(Clone, Copy)]
enum ExportProof {
    Playwright(Option<u32>),
    Alias(Option<u32>),
    Stars,
    Unknown,
}

pub(super) struct ExportLookup<'a> {
    resolver: &'a dyn ImportResolution,
    visible: &'a PathSet,
    proof: FxHashMap<ExportKey, ExportProof>,
    edges: Vec<CanonicalEdge<ExportKey, EdgeKind>>,
}

impl<'a> ExportLookup<'a> {
    pub(super) fn new(resolver: &'a dyn ImportResolution, visible: &'a PathSet) -> Self {
        Self {
            resolver,
            visible,
            proof: FxHashMap::default(),
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
            Export::Playwright(derived_at) => (ExportProof::Playwright(derived_at), Vec::new()),
            Export::Local(target, derived_at) => (ExportProof::Alias(derived_at), vec![target]),
            Export::Stars(targets) => (ExportProof::Stars, targets),
            Export::Unknown => (ExportProof::Unknown, Vec::new()),
        };
        next.extend(dependencies.iter().cloned());
        self.edges.extend(
            dependencies
                .into_iter()
                .map(|target| CanonicalEdge::new(key.clone(), target, EdgeKind::SymbolReexport)),
        );
        self.proof.insert(key, proof);
    }

    pub(super) fn local(&self, from: &Path, import: &ImportRef) -> Option<ExportKey> {
        let file = self.resolver.resolve(&import.specifier, from)?;
        (self.visible.contains(&file) && is_indexable(&file)).then(|| ExportKey {
            file,
            name: Arc::clone(&import.name),
        })
    }

    fn imported(&self, from: &Path, import: &ImportRef, derived_at: Option<u32>) -> Export {
        if import.is_playwright_test() {
            return Export::Playwright(derived_at);
        }
        self.local(from, import)
            .map_or(Export::Unknown, |target| Export::Local(target, derived_at))
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
                return self.imported(&key.file, &ImportRef::new(specifier, &binding.local), None);
            }
            if let Some(import) = facts
                .fixture_initializers
                .as_ref()
                .and_then(|fixtures| fixtures.0.get(&binding.local))
            {
                return self.imported(&key.file, &import.import, import.derived_at);
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
                // An external star can supply test but its other exports are opaque.
                return if ts.star_reexport_specifiers.len() == 1 {
                    Export::Playwright(None)
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
