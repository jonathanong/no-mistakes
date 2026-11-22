use super::{File, FxHashMap, ImportKind, Path, PathBuf};

pub(in crate::codebase::postgres::query_annotation::project) fn reachable<'a>(
    files: &'a FxHashMap<PathBuf, File<'_>>,
    roots: &[PathBuf],
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> crate::fx::FxHashSet<&'a Path> {
    let mut reached = crate::fx::fx_set();
    let mut pending = Vec::new();
    for root in roots {
        let root = files.get_key_value(root).unwrap().0.as_path();
        if reached.insert(root) {
            pending.push(root);
        }
    }
    while let Some(path) = pending.pop() {
        let module = &files[path].module;
        for import in &module.ts.imports {
            if import.kind == ImportKind::Type || import.computed {
                continue;
            }
            let Some(target) = module.target(&import.specifier, path, resolve) else {
                continue;
            };
            if let Some((target, _)) = files.get_key_value(target) {
                if reached.insert(target.as_path()) {
                    pending.push(target);
                }
            }
        }
    }
    reached
}
