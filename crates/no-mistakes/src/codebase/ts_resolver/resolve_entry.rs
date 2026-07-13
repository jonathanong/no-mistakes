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

/// Find the nearest `tsconfig.json` that is present in the request's canonical
/// visible-path candidates.
///
/// Callers should pass the candidates they already discovered for the request
/// so automatic config selection does not require a second repository scan.
#[doc(hidden)]
pub fn find_tsconfig_from_visible(start: &Path, visible_paths: &[PathBuf]) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        let candidate = normalize_path(&current.join("tsconfig.json"));
        if visible_paths
            .iter()
            .any(|path| normalize_path(path) == candidate)
        {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

/// Resolve the request's TypeScript configuration without consulting an
/// ignored auto-discovered `tsconfig.json`.
///
/// An explicit path remains authoritative, including when Git ignores it.
/// Automatic discovery is restricted to the caller's canonical visible path
/// candidates and otherwise falls back to an empty config anchored at `root`.
#[doc(hidden)]
pub fn resolve_tsconfig_from_visible(
    arg: Option<&Path>,
    root: &Path,
    visible_paths: &[PathBuf],
) -> Result<TsConfig> {
    if let Some(path) = arg {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            root.join(path)
        };
        return load_tsconfig(&path).context(format!("loading tsconfig {}", path.display()));
    }
    if let Some(candidate) = find_tsconfig_from_visible(root, visible_paths) {
        return load_tsconfig(&candidate).context(format!(
            "loading tsconfig {}",
            candidate.display()
        ));
    }
    Ok(TsConfig {
        dir: root.to_path_buf(),
        paths: Vec::new(),
        paths_dir: root.to_path_buf(),
        base_url: None,
    })
}

const EXTENSIONS: &[&str] = &[
    ".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx", ".cjs", ".cts", ".d.ts", ".d.mts", ".d.cts",
];

/// TypeScript source candidates for a NodeNext/ESM emitted extension.
/// Tried before the literal file; excludes `.jsx` and declaration files,
/// which may only appear after the literal file (see `emitted_fallback_candidates`).
fn emitted_source_candidates(extension: &str) -> &'static [&'static str] {
    match extension {
        "mjs" => &[".mts"],
        "cjs" => &[".cts"],
        "js" => &[".ts", ".tsx"],
        _ => &[],
    }
}

/// Declaration and `.jsx` fallbacks for a NodeNext/ESM emitted extension.
/// Tried only when neither a TypeScript source nor the literal file exists,
/// so the literal `.js`/`.mjs`/`.cjs` always takes priority over `.d.*` and `.jsx`.
fn emitted_fallback_candidates(extension: &str) -> &'static [&'static str] {
    match extension {
        "mjs" => &[".d.mts"],
        "cjs" => &[".d.cts"],
        "js" => &[".jsx", ".d.ts"],
        _ => &[],
    }
}
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

/// Load a `TsConfig` from `--tsconfig` if given, else search upward from `root`,
/// else return an empty config anchored at `root`.
///
/// Shared by every command that resolves import/re-export specifiers so the
/// `--tsconfig` fallback behaves identically across the CLI and N-API surface.
pub fn resolve_tsconfig(arg: Option<&Path>, root: &Path) -> Result<TsConfig> {
    let visible_paths = crate::codebase::ts_source::discover_visible_paths(root);
    resolve_tsconfig_from_visible(arg, root, &visible_paths)
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
