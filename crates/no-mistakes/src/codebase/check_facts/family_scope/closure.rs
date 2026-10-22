use super::FamilyPathScope;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::SourceStore;
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

const QUEUE_PACKAGES: &[&str] = &["bullmq", "glide-mq"];

pub(super) fn only_or_all(paths: BTreeSet<PathBuf>, universe: usize) -> FamilyPathScope {
    if paths.is_empty() || paths.len() >= universe {
        FamilyPathScope::All
    } else {
        FamilyPathScope::Only(Arc::new(paths))
    }
}

pub(super) fn with_import_closure(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    seeds: BTreeSet<PathBuf>,
) -> FamilyPathScope {
    if seeds.is_empty() {
        return FamilyPathScope::All;
    }
    let inventory: HashSet<PathBuf> = files.iter().map(|path| normalize_path(path)).collect();
    let mut seen = BTreeSet::new();
    let mut queue: VecDeque<PathBuf> = seeds.into_iter().collect();
    while let Some(path) = queue.pop_front() {
        let path = normalize_path(&path);
        if !seen.insert(path.clone()) {
            continue;
        }
        let Ok(source) = sources.read_path(&path) else {
            continue;
        };
        for specifier in relative_specifiers(&source) {
            if let Some(resolved) = resolve_relative(&path, &specifier, &inventory) {
                if !seen.contains(&resolved) {
                    queue.push_back(resolved);
                }
            }
        }
        if source_has_unresolved_local_alias(&source) {
            return FamilyPathScope::All;
        }
    }
    let _ = root;
    only_or_all(seen, files.len())
}

fn source_has_unresolved_local_alias(source: &str) -> bool {
    // Workspace packages (`@scope/pkg`) and tsconfig paths cannot be
    // distinguished from npm specifiers without the prepared resolver, so any
    // non-relative, non-queue specifier conservatively widens to All.
    quoted_specifiers(source)
        .any(|specifier| !specifier.starts_with('.') && !QUEUE_PACKAGES.contains(&specifier))
}

fn relative_specifiers(source: &str) -> Vec<String> {
    quoted_specifiers(source)
        .filter(|specifier| specifier.starts_with('.'))
        .map(str::to_string)
        .collect()
}

fn quoted_specifiers(source: &str) -> impl Iterator<Item = &str> {
    const PREFIXES: [&str; 8] = [
        "from '",
        "from \"",
        "import '",
        "import \"",
        "import('",
        "import(\"",
        "require('",
        "require(\"",
    ];
    PREFIXES.into_iter().flat_map(move |prefix| {
        let quote = prefix.as_bytes()[prefix.len() - 1] as char;
        let mut rest = source;
        let mut found = Vec::new();
        while let Some(at) = rest.find(prefix) {
            let after = &rest[at + prefix.len()..];
            if let Some(end) = after.find(quote) {
                found.push(&after[..end]);
            }
            rest = &rest[at + prefix.len()..];
        }
        found.into_iter()
    })
}

fn resolve_relative(from: &Path, specifier: &str, inventory: &HashSet<PathBuf>) -> Option<PathBuf> {
    let parent = from.parent()?;
    let base = normalize_path(&parent.join(specifier));
    let mut candidates = vec![base.clone()];
    for ext in ["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"] {
        candidates.push(base.with_extension(ext));
        candidates.push(base.join(format!("index.{ext}")));
    }
    candidates.into_iter().find(|path| inventory.contains(path))
}
