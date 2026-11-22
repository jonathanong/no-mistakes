use super::{ImportKind, Path, PathBuf, SyntaxModules};

pub(in crate::codebase::postgres::query_annotation::project) fn reachable<'a>(
    files: &SyntaxModules<'a>,
    roots: &[&'a Path],
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> crate::fx::FxHashSet<&'a Path> {
    let mut pending = roots.to_vec();
    let mut reached = roots.iter().copied().collect::<crate::fx::FxHashSet<_>>();
    while let Some(path) = pending.pop() {
        let module = files[path].0;
        for import in &module.ts.imports {
            if import.kind == ImportKind::Type || import.computed {
                continue;
            }
            let Some(target) = module.target(&import.specifier, path, resolve) else {
                continue;
            };
            if let Some((target, _)) = files.get_key_value(target) {
                if reached.insert(*target) {
                    pending.push(*target);
                }
            }
        }
    }
    reached
}
