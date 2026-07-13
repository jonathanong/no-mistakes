fn analyze_file_inner(
    path: &Path,
    root: &Path,
    visited: &mut HashSet<(PathBuf, bool, bool)>,
    fetches: &mut Vec<FetchOccurrence>,
    cache: &mut Cache,
    inherited: (bool, bool),
    visible_files: Option<&HashSet<PathBuf>>,
) -> Result<bool> {
    let (inherited_is_client, inherited_is_route_handler) = inherited;
    if !path.exists() {
        return Ok(false);
    }
    let abs_path = path.canonicalize()?;
    if visible_files.is_some_and(|visible| !visible.contains(&abs_path)) {
        return Ok(false);
    }
    let visit_key = (abs_path.clone(), inherited_is_client, inherited_is_route_handler);
    if visited.contains(&visit_key) {
        return Ok(false);
    }
    visited.insert(visit_key);
    let cache_key = (abs_path.clone(), inherited_is_client, inherited_is_route_handler);
    if let Some(cached_fetches) = cache.files.get(&cache_key) {
        fetches.extend(cached_fetches.fetches.clone());
        return Ok(cached_fetches.is_client);
    }
    let source = std::fs::read_to_string(&abs_path)?;
    let rel_file = relative_string(root, &abs_path);
    let mut file_fetches = Vec::new();
    let is_client = ast::with_program(path, &source, |program, _| -> Result<bool> {
        let has_use_server_directive = program.directives.iter().any(|directive| directive.directive == "use server");
        let has_use_client_directive = program.directives.iter().any(|directive| directive.directive == "use client");
        let is_client = !inherited_is_route_handler
            && !has_use_server_directive
            && (inherited_is_client || has_use_client_directive);
        let mut visitor = FetchVisitor::new(&source, rel_file.as_str(), is_client, inherited_is_route_handler);
        visitor.visit_program(program);
        file_fetches.extend(visitor.fetches);
        let referenced_identifiers = collect_identifier_references(program);
        let imports = match visible_files {
            Some(visible) => collect_runtime_imports_from_program_from_visible(
                &abs_path, program, &referenced_identifiers, visible,
            ),
            None => collect_runtime_imports_from_program(&abs_path, program, &referenced_identifiers),
        };
        for import in imports {
            analyze_file_inner(
                &import,
                root,
                visited,
                &mut file_fetches,
                cache,
                (is_client, inherited_is_route_handler),
                visible_files,
            )?;
        }
        Ok(is_client)
    })??;
    let cached = CachedFile { is_client, fetches: file_fetches.clone() };
    cache.files.insert(cache_key, cached.clone());
    if cached.is_client != inherited_is_client {
        cache.files.insert((abs_path, is_client, inherited_is_route_handler), cached);
    }
    fetches.extend(file_fetches);
    Ok(is_client)
}
