use super::{rust_sources, BTreeSet, Path, PathBuf};

pub(super) fn rule_ids_from_source(root: &Path) -> BTreeSet<String> {
    let rules_dir = root.join("crates/no-mistakes/src/codebase/rules");
    let mut sources = rust_sources(&rules_dir);
    sources.extend([
        root.join("crates/no-mistakes/src/codebase/unique_exports.rs"),
        root.join("crates/no-mistakes/src/playwright/rules.rs"),
    ]);

    sources
        .into_iter()
        .flat_map(|path| {
            read(&path)
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    let is_rule_constant = line.starts_with("pub const RULE_ID: &str = \"")
                        || line.starts_with("pub const REQUIRED_DOC_SECTION_RULE_ID: &str = \"")
                        || line.starts_with("pub const PLAYWRIGHT_");
                    is_rule_constant
                        .then(|| line.split_once("= \"")?.1.split_once('"').map(|(id, _)| id))
                        .flatten()
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

pub(super) fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub(super) fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}
