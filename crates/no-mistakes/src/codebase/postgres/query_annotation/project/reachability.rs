use super::File;
use crate::fx::{fx_map, fx_set, FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};

/// A root can emit an annotation event in another module through an imported
/// function, including a re-export chain. This request-local worklist answers
/// only whether the annotation evaluator could reach an executor under this
/// profile's resolver; it does not create dependency relationships or collect
/// new file facts. An iterative DFS memoizes only candidate booleans and
/// Tarjan traversal state; it does not build another relationship index. The
/// canonical graph is prepared after this annotation projection.
pub(super) fn roots_reaching_executors(
    files: &FxHashMap<PathBuf, File<'_>>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> FxHashSet<PathBuf> {
    if files.values().all(|file| file.executors.is_empty()) {
        return fx_set();
    }
    let mut index: FxHashMap<PathBuf, usize> = fx_map();
    let mut low: FxHashMap<PathBuf, usize> = fx_map();
    let mut active = fx_set();
    let mut components = Vec::new();
    let mut reaches_executor = fx_map();
    let mut known_true = fx_set();
    let mut frames = Vec::<Frame>::new();
    let mut roots = files.keys().cloned().collect::<Vec<_>>();
    roots.sort();
    for root in roots {
        if index.contains_key(&root) {
            continue;
        }
        enter(
            &root,
            &mut index,
            &mut low,
            &mut active,
            &mut components,
            &mut frames,
        );
        while let Some(frame) = frames.last_mut() {
            if let Some(neighbor) = next_neighbor(frame, files, resolve) {
                if !index.contains_key(&neighbor) {
                    enter(
                        &neighbor,
                        &mut index,
                        &mut low,
                        &mut active,
                        &mut components,
                        &mut frames,
                    );
                } else if active.contains(&neighbor) {
                    let path = &frame.path;
                    low.insert(path.clone(), low[path].min(index[&neighbor]));
                } else if reaches_executor[&neighbor] {
                    known_true.insert(frame.path.clone());
                }
                continue;
            }
            let path = frame.path.clone();
            if low[&path] == index[&path] {
                let mut members = Vec::new();
                loop {
                    let member = components.pop().expect("active SCC member");
                    active.remove(&member);
                    let done = member == path;
                    members.push(member);
                    if done {
                        break;
                    }
                }
                let can_reach = members.iter().any(|member| {
                    !files[member].executors.is_empty() || known_true.contains(member)
                });
                for member in members {
                    reaches_executor.insert(member, can_reach);
                }
            }
            frames.pop();
            if let Some(parent) = frames.last() {
                let parent_path = &parent.path;
                low.insert(parent_path.clone(), low[parent_path].min(low[&path]));
                if reaches_executor.get(&path).copied() == Some(true) {
                    known_true.insert(parent_path.clone());
                }
            }
        }
    }
    reaches_executor
        .into_iter()
        .filter_map(|(path, reaches)| reaches.then_some(path))
        .collect()
}

struct Frame {
    path: PathBuf,
    next_import: usize,
    next_export: usize,
    next_star: usize,
}

fn enter(
    path: &Path,
    index: &mut FxHashMap<PathBuf, usize>,
    low: &mut FxHashMap<PathBuf, usize>,
    active: &mut FxHashSet<PathBuf>,
    components: &mut Vec<PathBuf>,
    frames: &mut Vec<Frame>,
) {
    let path = path.to_path_buf();
    let next = index.len();
    index.insert(path.clone(), next);
    low.insert(path.clone(), next);
    active.insert(path.clone());
    components.push(path.clone());
    frames.push(Frame {
        path,
        next_import: 0,
        next_export: 0,
        next_star: 0,
    });
}

fn next_neighbor(
    frame: &mut Frame,
    files: &FxHashMap<PathBuf, File<'_>>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    let ts = files[&frame.path].ts;
    loop {
        // These are precisely the prepared binding paths that `name` and
        // `lookup_export` may follow. An unused binding is harmlessly included.
        let specifier = if let Some(binding) = ts.imported_bindings.get(frame.next_import) {
            frame.next_import += 1;
            Some(binding.specifier.as_str())
        } else if let Some(binding) = ts.exported_bindings.get(frame.next_export) {
            frame.next_export += 1;
            binding.specifier.as_deref()
        } else if let Some(specifier) = ts.star_reexport_specifiers.get(frame.next_star) {
            frame.next_star += 1;
            Some(specifier.as_str())
        } else {
            return None;
        };
        let Some(specifier) = specifier else { continue };
        let Some(target) = resolve(specifier, &frame.path) else {
            continue;
        };
        let target = crate::codebase::ts_resolver::normalize_path(&target);
        if files.contains_key(&target) {
            return Some(target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codebase::postgres::{extract_embedded_sql_from_program, EmbeddedSqlOptions};
    use crate::codebase::ts_source::facts::{
        collect_file_facts_from_program, TsFactContext, TsFactPlan,
    };

    #[test]
    fn skips_disconnected_fixture_modules_but_keeps_cyclic_reexports() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-reachability/src");
        let names = [
            "root.mts",
            "cycle-a.mts",
            "cycle-b.mts",
            "barrel.mts",
            "helper.mts",
            "db.mts",
            "idle.mts",
        ];
        let options = EmbeddedSqlOptions::configured("@app/db", &[]);
        let prepared = names
            .iter()
            .map(|name| {
                let path = crate::codebase::ts_resolver::normalize_path(&root.join(name));
                let source = std::fs::read_to_string(&path).unwrap();
                let (ts, calls) = crate::ast::with_program(&path, &source, |program, _| {
                    (
                        collect_file_facts_from_program(
                            &path,
                            TsFactPlan::imports(),
                            &TsFactContext::default(),
                            &source,
                            program,
                            None,
                            None,
                        ),
                        extract_embedded_sql_from_program(&path, program, &source, &options)
                            .call_starts,
                    )
                })
                .unwrap();
                (path, ts, calls)
            })
            .collect::<Vec<_>>();
        let empty =
            crate::codebase::postgres::query_annotation::QueryAnnotationFileFacts::default();
        let files = prepared
            .iter()
            .map(|(path, ts, calls)| {
                (
                    path.clone(),
                    File {
                        facts: &empty,
                        ts,
                        executors: calls.iter().copied().collect(),
                        imports: fx_map(),
                        exports: fx_map(),
                    },
                )
            })
            .collect::<FxHashMap<_, _>>();
        let roots = roots_reaching_executors(&files, &|specifier, from| {
            if specifier == "@app/db" {
                return Some(crate::codebase::ts_resolver::normalize_path(
                    &root.join("db.mts"),
                ));
            }
            let local = specifier.strip_prefix("./")?.replace(".mjs", ".mts");
            Some(crate::codebase::ts_resolver::normalize_path(
                &from.with_file_name(local),
            ))
        });
        for name in names
            .iter()
            .filter(|name| **name != "idle.mts" && **name != "db.mts")
        {
            assert!(
                roots.contains(&crate::codebase::ts_resolver::normalize_path(
                    &root.join(name)
                )),
                "{name}"
            );
        }
        assert!(
            !roots.contains(&crate::codebase::ts_resolver::normalize_path(
                &root.join("idle.mts")
            ))
        );
        assert!(
            !roots.contains(&crate::codebase::ts_resolver::normalize_path(
                &root.join("db.mts")
            ))
        );
        let only_local = roots_reaching_executors(&files, &|_, _| None);
        assert_eq!(
            only_local,
            FxHashSet::from_iter([crate::codebase::ts_resolver::normalize_path(
                &root.join("helper.mts")
            )])
        );
        let mut without_executors = files;
        for file in without_executors.values_mut() {
            file.executors.clear();
        }
        assert!(roots_reaching_executors(&without_executors, &|_, _| None).is_empty());
    }
}
