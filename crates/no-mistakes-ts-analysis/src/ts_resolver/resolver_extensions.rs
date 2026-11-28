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

pub fn has_explicit_non_source_extension(path: &Path) -> bool {
    has_explicit_extension(path) && !no_mistakes_ts_facts::extract::is_indexable(path)
}
