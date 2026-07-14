use crate::ast;
use crate::fetch::cache::{Cache, CachedFile};
use crate::fetch::file_facts::ParsedFileCache;
use crate::fetch::imports::{
    collect_identifier_references, collect_runtime_imports_from_program,
    collect_runtime_imports_from_program_from_visible,
};
use crate::fetch::resolve::relative_string;
use crate::fetch::types::FetchOccurrence;
use crate::fetch::types::FetchSide;
use crate::fetch::visitor::FetchVisitor;
use anyhow::Result;
use oxc_ast_visit::Visit;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn analyze_file(
    path: &Path,
    root: &Path,
    visited: &mut HashSet<(PathBuf, bool, bool)>,
    fetches: &mut Vec<FetchOccurrence>,
    cache: &mut Cache,
    inherited_is_client: bool,
    inherited_is_route_handler: bool,
) -> Result<bool> {
    analyze_file_inner(
        path,
        root,
        visited,
        fetches,
        cache,
        (inherited_is_client, inherited_is_route_handler),
        None,
    )
}

pub(crate) fn analyze_file_from_visible(
    path: &Path,
    root: &Path,
    visited: &mut HashSet<(PathBuf, bool, bool)>,
    fetches: &mut Vec<FetchOccurrence>,
    cache: &mut Cache,
    inherited: (bool, bool),
    visible_files: &HashSet<PathBuf>,
) -> Result<bool> {
    analyze_file_inner(
        path,
        root,
        visited,
        fetches,
        cache,
        inherited,
        Some(visible_files),
    )
}

pub(crate) struct VisibleFileAnalysis<'a> {
    pub root: &'a Path,
    pub visited: &'a mut HashSet<(PathBuf, bool, bool)>,
    pub fetches: &'a mut Vec<FetchOccurrence>,
    pub cache: &'a mut Cache,
    pub parsed_files: &'a mut ParsedFileCache,
    pub visible_files: &'a HashSet<PathBuf>,
}

pub(crate) fn analyze_file_from_visible_with_facts(
    path: &Path,
    inherited: (bool, bool),
    context: &mut VisibleFileAnalysis<'_>,
) -> Result<bool> {
    let VisibleFileAnalysis {
        root,
        visited,
        fetches,
        cache,
        parsed_files,
        visible_files,
    } = context;
    let (inherited_is_client, inherited_is_route_handler) = inherited;
    let abs_path = crate::codebase::ts_resolver::normalize_path(path);
    if !visible_files.contains(&abs_path) {
        return Ok(false);
    }
    let visit_key = (
        abs_path.clone(),
        inherited_is_client,
        inherited_is_route_handler,
    );
    if !visited.insert(visit_key.clone()) {
        return Ok(false);
    }
    if let Some(cached_fetches) = cache.files.get(&visit_key) {
        fetches.extend(cached_fetches.fetches.clone());
        return Ok(cached_fetches.is_client);
    }

    let facts = parsed_files.load(&abs_path, root, &mut cache.imports, visible_files)?;
    let is_client = !inherited_is_route_handler
        && !facts.has_use_server_directive
        && (inherited_is_client || facts.has_use_client_directive);
    let mut file_fetches = facts.fetches;
    for fetch in &mut file_fetches {
        fetch.side = if is_client {
            FetchSide::Client
        } else {
            FetchSide::Server
        };
        fetch.rsc = !is_client && !inherited_is_route_handler;
    }
    for import in facts.used_imports {
        analyze_file_from_visible_with_facts(
            &import,
            (is_client, inherited_is_route_handler),
            &mut VisibleFileAnalysis {
                root,
                visited,
                fetches: &mut file_fetches,
                cache,
                parsed_files,
                visible_files,
            },
        )?;
    }

    let cached = CachedFile {
        is_client,
        fetches: file_fetches.clone(),
    };
    cache.files.insert(visit_key, cached.clone());
    if cached.is_client != inherited_is_client {
        cache
            .files
            .insert((abs_path, is_client, inherited_is_route_handler), cached);
    }
    fetches.extend(file_fetches);
    Ok(is_client)
}

include!("file_analysis/legacy.rs");
