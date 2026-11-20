use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{KnownCallTargetRef, ResolvedCallSite};
use crate::codebase::ts_resolver::{
    normalize_path, ImportResolution, ScopedImportResolver, TsConfigCatalog,
    WorkspaceFallbackResolver,
};
use crate::codebase::workspaces::IndexedWorkspaceMap;
use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) struct ModuleCatalog<'a> {
    pub catalog: &'a TsConfigCatalog,
    pub sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    pub session: Option<&'a crate::codebase::analysis_session::AnalysisSession>,
}

pub(crate) fn module_catalog<'a>(
    catalog: Option<&'a TsConfigCatalog>,
    sources: Option<&'a crate::codebase::ts_source::SourceStore>,
    session: Option<&'a crate::codebase::analysis_session::AnalysisSession>,
) -> Option<ModuleCatalog<'a>> {
    catalog.map(|catalog| ModuleCatalog {
        catalog,
        sources,
        session,
    })
}

pub(super) struct ResolvedModules {
    files: BTreeMap<String, BTreeSet<PathBuf>>,
}

impl ResolvedModules {
    pub(super) fn empty() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub(super) fn files(&self, module: &str) -> Option<&BTreeSet<PathBuf>> {
        self.files.get(module)
    }
}

pub(super) fn resolve_modules(
    root: &Path,
    modules: impl IntoIterator<Item = String>,
    facts: &CheckFactMap,
    catalog: Option<ModuleCatalog<'_>>,
) -> Result<ResolvedModules> {
    let Some(catalog) = catalog else {
        return Ok(ResolvedModules::empty());
    };
    let modules: BTreeSet<String> = modules.into_iter().collect();
    let visible = facts
        .graph_file_universe()
        .iter()
        .map(|path| normalize_path(path))
        .collect::<crate::fx::PathSet>();
    let resolver = ScopedImportResolver::from_lookup(catalog.catalog, &visible, catalog.session);
    let workspace = match catalog.sources {
        Some(sources) => {
            crate::codebase::workspaces::load_indexed_from_source_store(root, sources)?
        }
        None => IndexedWorkspaceMap::default(),
    };
    let fallback = WorkspaceFallbackResolver::new(&resolver, &workspace, &visible);
    Ok(ResolvedModules {
        files: resolved_files(&fallback, &modules, facts),
    })
}

fn resolved_files(
    resolver: &dyn ImportResolution,
    modules: &BTreeSet<String>,
    facts: &CheckFactMap,
) -> BTreeMap<String, BTreeSet<PathBuf>> {
    let mut by_dir: BTreeMap<PathBuf, PathBuf> = BTreeMap::new();
    for file in facts.graph_file_universe() {
        let file = normalize_path(file);
        if let Some(dir) = file.parent() {
            by_dir.entry(dir.to_path_buf()).or_insert(file);
        }
    }
    let mut files = BTreeMap::new();
    for module in modules {
        let mut resolved = BTreeSet::new();
        for importer in by_dir.values() {
            if let Some(path) = resolver.resolve(module, importer) {
                resolved.insert(normalize_path(&path));
            }
        }
        files.insert(module.clone(), resolved);
    }
    files
}

pub(super) fn matches_resolved(
    files: &BTreeSet<PathBuf>,
    export: &str,
    site: &ResolvedCallSite,
) -> bool {
    if files.is_empty() || site.has_unknown_target() {
        return false;
    }
    let mut matched = false;
    for candidate in site.targets() {
        matched = true;
        let KnownCallTargetRef::ModuleExport {
            export_path,
            repository_target: Some((file, _)),
            ..
        } = candidate.target
        else {
            return false;
        };
        if export_path != export || !files.contains(&normalize_path(file)) {
            return false;
        }
    }
    matched
}
