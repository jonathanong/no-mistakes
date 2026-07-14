const HTTP_VERBS: &[&str] = &["get", "post", "put", "patch", "delete", "head", "options"];

/// Scan `source` for `<object>.route('<literal>').<method>(...)` chains.
/// Returns `(pattern, line_number)` pairs.
pub fn extract_backend_routes(source: &str, register_object: &str) -> Vec<(String, u32)> {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source, source_type).parse();
    extract_backend_routes_from_program(&ret.program, source, register_object)
}

pub fn extract_backend_routes_from_program<'a>(
    program: &Program<'a>,
    source: &str,
    register_object: &str,
) -> Vec<(String, u32)> {
    let mut results = Vec::new();

    for stmt in &program.body {
        collect_from_statement(stmt, source, register_object, true, &mut results);
    }

    results
}

/// Scan all `.mts`/`.ts` files under `dir` for backend route definitions using
/// `register_object` as the chain object. Returns `(file, pattern)` pairs.
pub fn collect_backend_routes_in_dir(
    dir: &std::path::Path,
    register_object: &str,
    pattern_globset: &globset::GlobSet,
) -> Vec<(PathBuf, String)> {
    let files: Vec<PathBuf> = crate::codebase::ts_source::discover_visible_paths(dir)
        .into_iter()
        .filter(|path| {
            path.strip_prefix(dir)
                .ok()
                .and_then(std::path::Path::parent)
                .is_none_or(|parent| {
                    parent.components().all(|component| {
                        !crate::codebase::ts_source::is_skipped_dir(
                            component.as_os_str().to_str().unwrap_or_default(),
                        )
                    })
                })
        })
        .collect();
    collect_backend_routes_from_files(dir, &files, register_object, pattern_globset)
}

/// Collect backend route definitions from an already-discovered file list.
pub fn collect_backend_routes_from_files(
    root: &std::path::Path,
    files: &[PathBuf],
    register_object: &str,
    pattern_globset: &globset::GlobSet,
) -> Vec<(PathBuf, String)> {
    let mut results = Vec::new();

    for path in files {
        if !path.is_file() {
            continue;
        }
        let rel = match path.strip_prefix(root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if !pattern_globset.is_match(rel) {
            continue;
        }
        let source = std::fs::read_to_string(path).unwrap_or_default();
        for (route, _line) in extract_backend_routes(&source, register_object) {
            results.push((path.clone(), route));
        }
    }

    results
}
