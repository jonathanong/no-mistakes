pub fn find_tsconfig(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = current.join("tsconfig.json");
        if candidate.exists() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

const EXTENSIONS: &[&str] = &[".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx", ".cjs", ".cts"];
const EXPLICIT_EXTENSIONS: &[&str] = &[
    "mts", "ts", "tsx", "mjs", "js", "jsx", "cjs", "cts", "json", "css", "scss", "sass", "less",
    "svg", "png", "jpg", "jpeg", "gif", "webp", "avif", "txt", "wasm",
];

fn has_explicit_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| EXPLICIT_EXTENSIONS.contains(&ext))
        .unwrap_or(false)
}

/// Resolve `specifier` (as it appears in an import in `importing_file`) to an
/// absolute path on disk. Returns `None` for bare npm specifiers or if no file
/// is found.
///
/// Resolution order:
/// 1. Relative (`./` or `../`): join with importer's directory, try extension candidates.
/// 2. tsconfig path alias: match against `paths` map, substitute capture, try candidates.
/// 3. None.
pub fn resolve_import(
    specifier: &str,
    importing_file: &Path,
    tsconfig: &TsConfig,
) -> Option<PathBuf> {
    ImportResolver::new(tsconfig)
        .without_cache()
        .resolve(specifier, importing_file)
}

