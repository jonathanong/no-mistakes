use super::{demand, File, Module};
use crate::codebase::check_facts::CheckFileFacts;
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::FileIdMap;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct Profile<'a> {
    pub options: EmbeddedSqlOptions,
    pub files: FxHashMap<PathBuf, File<'a>>,
    pub roots: Vec<PathBuf>,
    pub untrusted_array_intrinsics: bool,
}

pub(super) fn profiles<'a>(
    files: &'a FileIdMap<CheckFileFacts>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<Profile<'a>> {
    let mut profiles = files
        .into_iter()
        .flat_map(|(_, file)| {
            file.query_annotation
                .iter()
                .map(|(options, _)| options.clone())
        })
        .collect::<Vec<_>>();
    profiles.sort();
    profiles.dedup();
    let modules = files
        .into_iter()
        .filter(|(_, file)| file.parse_error.is_none() && !file.query_annotation.is_empty())
        .map(|(path, file)| (path.clone(), Arc::new(Module::new(Arc::clone(&file.ts)))))
        .collect::<FxHashMap<_, _>>();
    profiles
        .into_iter()
        .map(|options| {
            let collected = files
                .into_iter()
                .filter_map(|(path, file)| {
                    if file.parse_error.is_some() {
                        return None;
                    }
                    let (_, facts) = file
                        .query_annotation
                        .iter()
                        .find(|(profile, _)| profile == &options)?;
                    let (_, embedded) = file
                        .embedded_sql
                        .iter()
                        .find(|(profile, _)| profile == &options)
                        .expect("annotation summaries borrow their embedded profile");
                    Some((
                        normalize_path(path),
                        File {
                            module: Arc::clone(&modules[path]),
                            facts,
                            executors: embedded.call_spans.iter().copied().collect(),
                        },
                    ))
                })
                .collect::<FxHashMap<_, _>>();
            let untrusted_array_intrinsics = collected
                .values()
                .any(|file| file.facts.array_prototype_exposed);
            let roots = demand::roots(&collected, &options, resolve);
            if let Some(observer) = crate::diagnostics::current() {
                observer.increment("postgres.annotation.roots", roots.len() as u64);
            }

            let roots = roots.into_iter().map(Path::to_path_buf).collect();
            Profile {
                options,
                files: collected,
                roots,
                untrusted_array_intrinsics,
            }
        })
        .collect()
}
