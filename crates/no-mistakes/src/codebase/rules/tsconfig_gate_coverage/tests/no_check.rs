use super::*;

#[test]
fn resolution_defers_ambiguous_configs_and_resolves_local_extensionless_bases() {
    let root = fixture_root("no-check-edge-cases");
    let paths = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = super::super::super::source_store_for_files(&paths);
    // Malformed, cyclic, missing, and package-based extends deliberately defer to tsc.
    let tracked = BTreeSet::from([
        "bad-array/tsconfig.json".to_string(),
        "bad-compiler-options/tsconfig.json".to_string(),
        "bad-extends/tsconfig.json".to_string(),
        "bad-no-check/tsconfig.json".to_string(),
        "cycle/tsconfig.json".to_string(),
        "directory-base/tsconfig.json".to_string(),
        "empty/tsconfig.json".to_string(),
        "file-base/tsconfig.json".to_string(),
        "missing-base/tsconfig.json".to_string(),
        "package-base/tsconfig.json".to_string(),
    ]);

    assert_eq!(
        non_enforcing_tsconfigs(&root, &tracked, &sources),
        BTreeSet::from([
            "directory-base/tsconfig.json".to_string(),
            "file-base/tsconfig.json".to_string(),
        ])
    );
}
