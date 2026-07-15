pub fn bfs_reachable(entrypoint: &Path, tsconfig: &ts_resolver::TsConfig) -> HashSet<PathBuf> {
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut queue: VecDeque<PathBuf> = VecDeque::new();

    let start = entrypoint
        .canonicalize()
        .unwrap_or(entrypoint.to_path_buf());
    queue.push_back(start);

    while let Some(file) = queue.pop_front() {
        if visited.contains(&file) {
            continue;
        }
        visited.insert(file.clone());

        let source = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for specifier in collect_import_specifiers(&source) {
            if let Some(resolved) = ts_resolver::resolve_import(&specifier, &file, tsconfig) {
                let canonical = resolved.canonicalize().unwrap_or(resolved);
                if !visited.contains(&canonical) {
                    queue.push_back(canonical);
                }
            }
        }
    }

    visited
}

/// Parse `source` and collect all import/export module specifier strings.
pub fn collect_import_specifiers(source: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = crate::ast::parse(
        Path::new("queue-reachability.ts"),
        &allocator,
        source,
        source_type,
    );
    let mut specifiers = Vec::new();

    for stmt in &ret.program.body {
        match stmt {
            Statement::ImportDeclaration(import_decl) => {
                specifiers.push(import_decl.source.value.to_string());
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(src) = &export.source {
                    specifiers.push(src.value.to_string());
                }
            }
            Statement::ExportAllDeclaration(export) => {
                specifiers.push(export.source.value.to_string());
            }
            _ => {}
        }
    }

    specifiers
}
