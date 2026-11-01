use super::ScriptEntries;
use crate::codebase::ts_resolver::normalize_path;
use std::path::{Path, PathBuf};

fn fixture(case: &str) -> PathBuf {
    normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-exports-only-used-by-tests/fixture")
            .join(case),
    )
}

fn relative_entries(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let found = ScriptEntries::collect(root, files);
    let mut names: Vec<_> = found
        .paths
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    names.sort();
    names
}

#[test]
fn bundler_scripts_resolve_positional_entries_and_skip_outputs() {
    let root = fixture("script-entry-shapes");
    let nested = root.join("packages/nested/src/nested.mts");
    assert_eq!(
        relative_entries(&root, &[nested]),
        [
            "packages/nested/src/nested.mts",
            "src/and.mts",
            "src/env.mts",
            "src/esbuild.mts",
            "src/nl.mts",
            "src/or.mts",
            "src/quoted name.mts",
            "src/rollup.mts",
            "src/semi.mts",
            "src/tsup.mts",
            "src/vite.mts",
            "src/webpack.mts",
            "src/wrangler.mts",
        ]
    );
}

#[test]
fn malformed_and_scriptless_manifests_contribute_no_entries() {
    assert!(relative_entries(&fixture("script-entry-malformed"), &[]).is_empty());
    assert!(relative_entries(&fixture("script-entry-noscripts"), &[]).is_empty());
    assert!(relative_entries(&fixture("script-entry").join("src"), &[]).is_empty());
}
