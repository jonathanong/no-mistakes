use crate::playwright::analysis::app_collect::collect_selector_source_files;
use crate::playwright::config;
use crate::playwright::fsutil::build_globset;
use crate::playwright::routes;
use anyhow::Result;
use dashmap::DashMap;
use oxc_ast::ast::{ImportOrExportKind, Statement};
use rayon::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod dynamic_imports;

pub(crate) fn collect_route_reachable_files(
    root: &Path,
    settings: &config::Settings,
    routes: &[routes::Route],
) -> Result<BTreeMap<Arc<String>, BTreeSet<Arc<String>>>> {
    let include = build_globset(&settings.selector_include)?;
    let exclude = build_globset(&settings.selector_exclude)?;
    let include_all = settings.selector_include.is_empty();
    let selector_files =
        collect_selector_source_files(root, settings, &include, &exclude, include_all);
    let selector_rel_by_file: HashMap<_, _> = selector_files
        .iter()
        .map(|file| {
            (
                crate::codebase::ts_resolver::normalize_path(file),
                Arc::new(crate::playwright::fsutil::relative_string(root, file)),
            )
        })
        .collect();
    let frontend_root = root.join(&settings.frontend_root);
    let tsconfig = crate::codebase::ts_resolver::find_tsconfig(&frontend_root)
        .or_else(|| crate::codebase::ts_resolver::find_tsconfig(root))
        .map(|path| crate::codebase::ts_resolver::load_tsconfig(&path))
        .transpose()?
        .unwrap_or_else(|| crate::codebase::ts_resolver::TsConfig {
            dir: root.to_path_buf(),
            paths: Vec::new(),
            paths_dir: root.to_path_buf(),
            base_url: None,
        });
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let import_cache = DashMap::new();
    let route_reachable_files = routes
        .par_iter()
        .map(|route| {
            Ok((
                route_key(root, &route.file),
                reachable_files(
                    root,
                    settings,
                    &route.file,
                    &selector_rel_by_file,
                    &resolver,
                    &import_cache,
                )?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(route_reachable_files)
}

fn reachable_files(
    root: &Path,
    settings: &config::Settings,
    route_file: &Path,
    selector_rel_by_file: &HashMap<std::path::PathBuf, Arc<String>>,
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
    import_cache: &DashMap<PathBuf, Arc<Vec<PathBuf>>>,
) -> Result<BTreeSet<Arc<String>>> {
    let mut reachable = BTreeSet::new();
    let mut stack = route_entry_files(root, settings, route_file)
        .into_iter()
        .map(|file| crate::codebase::ts_resolver::normalize_path(&file))
        .collect::<Vec<_>>();
    let mut seen = HashSet::new();
    while let Some(file) = stack.pop() {
        if !seen.insert(file.clone()) {
            continue;
        }
        if let Some(rel) = selector_rel_by_file.get(&file) {
            reachable.insert(rel.clone());
        }
        let imports = collect_route_imports(&file, resolver, import_cache)?;
        stack.extend(
            imports
                .iter()
                .map(|file| crate::codebase::ts_resolver::normalize_path(file)),
        );
    }
    Ok(reachable)
}

fn route_entry_files(root: &Path, settings: &config::Settings, route_file: &Path) -> Vec<PathBuf> {
    let frontend_root = root.join(&settings.frontend_root);
    let mut files = vec![route_file.to_path_buf()];
    files.extend(
        crate::fetch::import_routes::collect_layout_chain_files(route_file, &frontend_root)
            .into_iter()
            .filter(|file| {
                matches!(
                    file.file_stem().and_then(|stem| stem.to_str()),
                    Some("layout" | "template")
                )
            }),
    );
    files
}

fn collect_route_imports(
    path: &Path,
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
    import_cache: &DashMap<PathBuf, Arc<Vec<PathBuf>>>,
) -> Result<Arc<Vec<PathBuf>>> {
    let abs_path = path.canonicalize()?;
    if let Some(cached_imports) = import_cache.get(&abs_path) {
        return Ok(cached_imports.value().clone());
    }
    if !is_script_path(&abs_path) {
        let imports = Arc::new(Vec::new());
        import_cache.insert(abs_path, imports.clone());
        return Ok(imports);
    }

    let source = std::fs::read_to_string(&abs_path)?;
    let imports = crate::ast::with_program(&abs_path, &source, |program, _source| {
        collect_route_imports_from_program(&abs_path, program, resolver)
    })?;
    let imports = Arc::new(imports);
    import_cache.insert(abs_path, imports.clone());
    Ok(imports)
}

fn collect_route_imports_from_program<'a>(
    abs_path: &Path,
    program: &oxc_ast::ast::Program<'a>,
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
) -> Vec<PathBuf> {
    let mut imports = Vec::new();
    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(import)
                if crate::fetch::import_shape::is_runtime_import(import) =>
            {
                if let Some(resolved) = resolver.resolve(import.source.value.as_str(), abs_path) {
                    imports.push(resolved);
                }
            }
            Statement::ExportNamedDeclaration(export) => {
                if !crate::fetch::import_shape::is_runtime_export(export) {
                    continue;
                }
                if let Some(source) = &export.source {
                    if let Some(resolved) = resolver.resolve(source.value.as_str(), abs_path) {
                        imports.push(resolved);
                    }
                }
            }
            Statement::ExportAllDeclaration(export) => {
                if export.export_kind == ImportOrExportKind::Type {
                    continue;
                }
                if let Some(resolved) = resolver.resolve(export.source.value.as_str(), abs_path) {
                    imports.push(resolved);
                }
            }
            _ => {}
        }
    }
    imports.extend(dynamic_imports::collect(abs_path, program, resolver));
    imports
}

fn route_key(root: &Path, file: &Path) -> Arc<String> {
    Arc::new(crate::playwright::fsutil::relative_string(root, file))
}

fn is_script_path(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("js" | "jsx" | "ts" | "tsx" | "mjs" | "mts" | "cjs" | "cts")
    )
}

#[cfg(test)]
mod tests;
