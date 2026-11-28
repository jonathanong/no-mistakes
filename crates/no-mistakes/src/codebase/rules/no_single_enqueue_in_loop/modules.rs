use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::graph::{KnownCallTargetRef, ResolvedCallSite};
use crate::codebase::ts_resolver::{
    normalize_path, ImportResolution, ScopedImportResolver, TsConfigCatalog,
    WorkspaceFallbackResolver,
};
use crate::codebase::workspace_resolution::IndexedWorkspaceMap;
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

pub(in crate::codebase::rules) struct ResolvedModules {
    files: BTreeMap<String, BTreeSet<PathBuf>>,
}

impl ResolvedModules {
    pub(super) fn empty() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub(in crate::codebase::rules) fn files(&self, module: &str) -> Option<&BTreeSet<PathBuf>> {
        self.files.get(module)
    }

    #[cfg(test)]
    pub(in crate::codebase::rules) fn for_test(module: &str, files: BTreeSet<PathBuf>) -> Self {
        let mut mapped = BTreeMap::new();
        mapped.insert(module.to_string(), files);
        Self { files: mapped }
    }
}

pub(in crate::codebase::rules) fn resolve_modules(
    root: &Path,
    modules: impl IntoIterator<Item = String>,
    facts: &CheckFactMap,
    catalog: Option<ModuleCatalog<'_>>,
) -> Result<ResolvedModules> {
    let Some(catalog) = catalog else {
        return Ok(ResolvedModules::empty());
    };
    let modules: BTreeSet<String> = modules.into_iter().collect();
    if modules.is_empty() {
        return Ok(ResolvedModules::empty());
    }
    let visible = facts
        .graph_file_universe()
        .iter()
        .map(|path| normalize_path(path))
        .collect::<crate::fx::PathSet>();
    let resolver = ScopedImportResolver::from_lookup(
        catalog.catalog,
        &visible,
        catalog
            .session
            .map(|session| session as &dyn crate::codebase::ts_resolver::ResolverSession),
    );
    let workspace = match catalog.sources {
        Some(sources) => {
            crate::codebase::package_analysis::load_indexed_from_source_store(root, sources)?
        }
        None => IndexedWorkspaceMap::default(),
    };
    let fallback = WorkspaceFallbackResolver::new(&resolver, &workspace, &visible);
    Ok(ResolvedModules {
        files: resolved_files(&fallback, catalog.catalog, &modules, facts),
    })
}

fn resolved_files(
    resolver: &dyn ImportResolution,
    catalog: &TsConfigCatalog,
    modules: &BTreeSet<String>,
    facts: &CheckFactMap,
) -> BTreeMap<String, BTreeSet<PathBuf>> {
    let mut seen = BTreeSet::new();
    let mut importers = Vec::new();
    for file in facts.graph_file_universe() {
        let file = normalize_path(file);
        let Some(dir) = file.parent() else {
            continue;
        };
        let scope = catalog.resolver_scope_index_for(&file);
        if seen.insert((dir.to_path_buf(), scope)) {
            importers.push(file);
        }
    }
    let mut files = BTreeMap::new();
    for module in modules {
        let mut resolved = BTreeSet::new();
        for importer in &importers {
            if let Some(path) = resolver.resolve(module, importer) {
                resolved.insert(normalize_path(&path));
            }
        }
        files.insert(module.clone(), resolved);
    }
    files
}

pub(in crate::codebase::rules) fn matches_resolved(
    files: &BTreeSet<PathBuf>,
    export: &str,
    site: &ResolvedCallSite,
) -> bool {
    if files.is_empty() || site.has_unknown_target() || site.targets().is_empty() {
        return false;
    }
    site.targets()
        .all(|candidate| matches_resolved_candidate(files, export, candidate.target))
}

pub(in crate::codebase::rules) fn matches_resolved_candidate(
    files: &BTreeSet<PathBuf>,
    export: &str,
    target: KnownCallTargetRef<'_>,
) -> bool {
    matches!(target, KnownCallTargetRef::ModuleExport {
        export_path,
        repository_target: Some((file, _)),
        ..
    } if export_path == export && files.contains(&normalize_path(file)))
}
