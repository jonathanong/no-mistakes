use super::{resolve_mounts_with_resolver, FileFacts, ResolvedMount};
use crate::codebase::ts_resolver::ImportResolver;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub(crate) fn resolve_mounts(facts: &HashMap<PathBuf, FileFacts>) -> Vec<ResolvedMount> {
    let visible = facts.keys().cloned().collect::<HashSet<_>>();
    let root = facts
        .keys()
        .filter_map(|path| path.parent())
        .min_by_key(|path| path.components().count())
        .unwrap_or(Path::new(""));
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.to_path_buf(),
        paths_dir: root.to_path_buf(),
        paths: Vec::new(),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    resolve_mounts_with_resolver(facts, &resolver)
}
