use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::ts_resolver::ImportResolver;
use std::path::{Path, PathBuf};

/// Declaration-file suffixes tried for a type-only import, in TypeScript's
/// preference order (direct file, then directory index).
const DECLARATION_SUFFIXES: &[&str] = &[
    ".d.ts",
    ".d.mts",
    ".d.cts",
    "/index.d.ts",
    "/index.d.mts",
    "/index.d.cts",
];

/// Strip an emitted `.js`/`.mjs`/`.cjs` extension so declaration candidates are
/// built from the module stem (`./types.js` → `./types`).
fn module_stem(specifier: &str) -> &str {
    for ext in [".js", ".mjs", ".cjs"] {
        if let Some(stem) = specifier.strip_suffix(ext) {
            return stem;
        }
    }
    specifier
}

/// Resolve a type-only import to a declaration file or declaration index when
/// no emitted module exists. Returns `None` for value imports.
pub(super) fn resolve_declaration(
    imp: &ExtractedImport,
    abs_file: &Path,
    resolver: &ImportResolver,
) -> Option<PathBuf> {
    if imp.kind != ImportKind::Type {
        return None;
    }
    let stem = module_stem(&imp.specifier);
    DECLARATION_SUFFIXES
        .iter()
        .find_map(|suffix| resolver.resolve(&format!("{stem}{suffix}"), abs_file))
}

/// Map an emitted-extension specifier to its TypeScript source, as NodeNext/ESM
/// projects require (`./util.js` → `./util.ts`). Returns `None` for specifiers
/// without an emitted JS extension.
pub(super) fn resolve_ts_source(
    specifier: &str,
    abs_file: &Path,
    resolver: &ImportResolver,
) -> Option<PathBuf> {
    let (js_ext, source_exts): (&str, &[&str]) = if specifier.ends_with(".js") {
        (".js", &[".ts", ".tsx"])
    } else if specifier.ends_with(".mjs") {
        (".mjs", &[".mts"])
    } else if specifier.ends_with(".cjs") {
        (".cjs", &[".cts"])
    } else {
        return None;
    };
    let stem = &specifier[..specifier.len() - js_ext.len()];
    source_exts
        .iter()
        .find_map(|ext| resolver.resolve(&format!("{stem}{ext}"), abs_file))
}
