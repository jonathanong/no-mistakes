use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
use crate::codebase::ts_resolver::ImportResolver;
use std::path::{Path, PathBuf};

/// TypeScript source and declaration candidates for a specifier, by its emitted
/// extension. NodeNext keeps `.mjs`/`.cjs` mode-specific; a bare specifier tries
/// every declaration form.
fn candidates(specifier: &str) -> (&str, &'static [&'static str], &'static [&'static str]) {
    if let Some(stem) = specifier.strip_suffix(".mjs") {
        (stem, &[".mts"], &[".d.mts"])
    } else if let Some(stem) = specifier.strip_suffix(".cjs") {
        (stem, &[".cts"], &[".d.cts"])
    } else if let Some(stem) = specifier.strip_suffix(".js") {
        (stem, &[".ts", ".tsx", ".jsx"], &[".d.ts"])
    } else {
        (
            specifier,
            &[],
            &[
                ".d.ts",
                ".d.mts",
                ".d.cts",
                "/index.d.ts",
                "/index.d.mts",
                "/index.d.cts",
            ],
        )
    }
}

/// Map an emitted-extension specifier to its TypeScript source, as NodeNext/ESM
/// projects require (`./util.js` → `./util.ts`). `None` for bare specifiers.
pub(super) fn resolve_ts_source(
    specifier: &str,
    abs_file: &Path,
    resolver: &ImportResolver,
) -> Option<PathBuf> {
    let (stem, source_exts, _) = candidates(specifier);
    source_exts
        .iter()
        .find_map(|ext| resolver.resolve(&format!("{stem}{ext}"), abs_file))
}

/// Resolve a type-only import to a declaration file or declaration index when no
/// emitted module exists. `None` for value imports.
pub(super) fn resolve_declaration(
    imp: &ExtractedImport,
    abs_file: &Path,
    resolver: &ImportResolver,
) -> Option<PathBuf> {
    if imp.kind != ImportKind::Type {
        return None;
    }
    let (stem, _, declaration_suffixes) = candidates(&imp.specifier);
    declaration_suffixes
        .iter()
        .find_map(|suffix| resolver.resolve(&format!("{stem}{suffix}"), abs_file))
}

/// True for a `.d.ts`/`.d.mts`/`.d.cts` declaration file, which only satisfies
/// type-only references (no emitted runtime module).
pub(super) fn is_declaration_file(path: &Path) -> bool {
    let name = path.to_string_lossy();
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}
