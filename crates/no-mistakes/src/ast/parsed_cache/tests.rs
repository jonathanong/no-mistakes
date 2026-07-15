use super::ParsedProgramCache;
use std::path::Path;

pub(in crate::ast) fn len(cache: &ParsedProgramCache) -> usize {
    cache.entries.borrow().len()
}

#[test]
fn cached_parse_errors_are_available_without_reparsing() {
    let cache = ParsedProgramCache::default();
    let path = Path::new("unsupported.runner-config");

    let error = cache.with_program(path, "", |_, _| ()).unwrap_err();

    assert_eq!(cache.parse_error(path).as_deref(), Some(error.as_str()));
    assert!(cache.parse_error(Path::new("not-cached.ts")).is_none());
}

#[test]
fn legacy_symbols_share_only_ordinary_typescript_cache_entries() {
    for path in ["source.ts", "source.tsx"] {
        assert!(
            super::legacy_symbols_share_standard_parse(Path::new(path)),
            "{path}"
        );
    }
    for path in [
        "source.js",
        "source.jsx",
        "source.mjs",
        "source.cjs",
        "source.mts",
        "source.cts",
        "source.d.ts",
        "source.d.mts",
        "source.d.cts",
        "index.d.css.ts",
    ] {
        assert!(
            !super::legacy_symbols_share_standard_parse(Path::new(path)),
            "{path}"
        );
    }
}

#[test]
fn legacy_symbols_reuse_or_split_physical_cache_by_source_semantics() {
    for (path, expected_entries) in [
        ("source.ts", 1),
        ("source.tsx", 1),
        ("source.js", 2),
        ("source.mts", 2),
        ("source.cts", 2),
        ("source.d.ts", 2),
    ] {
        let cache = ParsedProgramCache::default();
        let path = Path::new(path);
        cache
            .with_recovered_program_observed(path, "export const value = 1;", || {}, |_, _, _| ())
            .unwrap();
        cache
            .with_legacy_symbols_program_observed(
                path,
                "export const value = 1;",
                || {},
                |_, _, _| (),
            )
            .unwrap();
        assert_eq!(len(&cache), expected_entries, "{}", path.display());
    }
}
