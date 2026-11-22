use super::{demand, File, Module};
use crate::codebase::check_facts::{
    sql_source_prerequisites::{
        source_demanded, source_error, FailedSqlSources, SqlProjection, SqlSourceKey,
    },
    CheckFileFacts,
};
use crate::codebase::postgres::EmbeddedSqlOptions;
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::FileIdMap;
use crate::fx::{FxHashMap, FxHashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) struct Profile<'a> {
    pub options: EmbeddedSqlOptions,
    pub files: FxHashMap<PathBuf, File<'a>>,
    pub roots: Vec<PathBuf>,
    pub untrusted_array_intrinsics: bool,
    pub failed_sources: Arc<FailedSqlSources>,
}

pub(crate) struct Catalog {
    pub(super) options: Vec<EmbeddedSqlOptions>,
    modules: FxHashMap<PathBuf, Arc<Module>>,
    pub(super) syntax_roots: FxHashMap<EmbeddedSqlOptions, FxHashSet<PathBuf>>,
}

pub(super) fn catalog(files: &FileIdMap<CheckFileFacts>) -> Catalog {
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
    let module_ids = Arc::new(super::super::module_ids::ModuleIds::new(
        files
            .into_iter()
            .filter(|(_, file)| file.parse_error.is_none() && !file.query_annotation.is_empty())
            .map(|(path, _)| normalize_path(path)),
    ));
    let modules = crate::perf_trace::trace("postgres.annotation.prepare.modules", || {
        files
            .into_iter()
            .filter(|(_, file)| file.parse_error.is_none() && !file.query_annotation.is_empty())
            .map(|(path, file)| {
                (
                    path.clone(),
                    Arc::new(
                        Module::new(Arc::clone(&file.ts)).with_module_ids(Arc::clone(&module_ids)),
                    ),
                )
            })
            .collect::<FxHashMap<_, _>>()
    });
    Catalog {
        options: profiles,
        modules,
        syntax_roots: FxHashMap::default(),
    }
}

pub(super) fn syntax_modules<'a>(
    files: &FileIdMap<CheckFileFacts>,
    catalog: &'a Catalog,
    options: &EmbeddedSqlOptions,
) -> demand::SyntaxModules<'a> {
    catalog
        .modules
        .iter()
        .filter_map(|(path, module)| {
            let (_, facts) = files
                .get(path)?
                .query_annotation
                .iter()
                .find(|(profile, _)| profile == options)?;
            Some((
                path.as_path(),
                (module.as_ref(), facts.syntax_in_scope != Some(false)),
            ))
        })
        .collect()
}

pub(super) fn profiles_with_catalog<'a>(
    files: &'a FileIdMap<CheckFileFacts>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
    catalog: &Catalog,
) -> Vec<Profile<'a>> {
    let failed_files = files
        .into_iter()
        .filter(|(_, file)| source_error(file).is_some())
        .map(|(path, _)| path)
        .collect::<Vec<_>>();
    let empty_failures = Arc::<FailedSqlSources>::default();
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment("postgres.annotation.profiles", catalog.options.len() as u64);
    }
    catalog
        .options
        .iter()
        .cloned()
        .map(|options| {
            let failed_sources = if failed_files.is_empty() {
                Arc::clone(&empty_failures)
            } else {
                let key = SqlSourceKey::profile(SqlProjection::AnnotationMax, &options);
                Arc::new(
                    failed_files
                        .iter()
                        .filter(|path| source_demanded(files, path, &key))
                        .map(|path| normalize_path(path))
                        .collect(),
                )
            };
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
                            module: Arc::clone(&catalog.modules[path]),
                            facts,
                            executors: embedded.call_spans.iter().copied().collect(),
                        },
                    ))
                })
                .collect::<FxHashMap<_, _>>();
            let untrusted_array_intrinsics = collected
                .values()
                .any(|file| file.facts.exposes_array_prototype());
            let roots = crate::perf_trace::trace("postgres.annotation.prepare.roots", || {
                let allowed = catalog.syntax_roots.get(&options);
                if allowed.is_some_and(FxHashSet::is_empty) {
                    return Vec::new();
                }
                let mut roots = demand::roots(&collected, resolve);
                if let Some(allowed) = allowed {
                    roots.retain(|path| allowed.contains(*path));
                }
                roots
            });
            if let Some(observer) = &observer {
                observer.increment("postgres.annotation.roots", roots.len() as u64);
                super::work::roots(observer, &collected, &roots, resolve);
            }
            let roots = roots.into_iter().map(Path::to_path_buf).collect();
            Profile {
                options,
                files: collected,
                roots,
                untrusted_array_intrinsics,
                failed_sources,
            }
        })
        .collect()
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
pub(super) fn profiles<'a>(
    files: &'a FileIdMap<CheckFileFacts>,
    resolve: &impl Fn(&str, &Path) -> Option<PathBuf>,
) -> Vec<Profile<'a>> {
    profiles_with_catalog(files, resolve, &catalog(files))
}
