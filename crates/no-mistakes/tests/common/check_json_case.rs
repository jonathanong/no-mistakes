use std::path::PathBuf;

/// One snapshot case: `root` is a saved fixture directory relative to the
/// repository root and `config` is resolved against that directory.
pub fn case(label: &'static str, root: &str, config: &str) -> (&'static str, PathBuf, PathBuf) {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = no_mistakes::codebase::ts_resolver::normalize_path(&repo.join(root));
    let config = no_mistakes::codebase::ts_resolver::normalize_path(&root.join(config));
    (label, root, config)
}
