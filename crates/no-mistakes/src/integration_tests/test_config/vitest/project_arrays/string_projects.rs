use super::{import_bindings, objects, shared, top_level_function_bodies, Ctx, Options};
use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod config_paths;
mod roots;
use config_paths::is_runtime_project_source;
pub(super) use config_paths::is_vitest_project_config;
pub(super) use roots::string_project_roots;
pub(in crate::integration_tests::test_config::vitest) use roots::string_project_roots_with_resolver;

/// Parse static project config files through the same object/config extractor
/// as inline projects; never execute the config.
pub(super) fn string_project_options_for_paths(
    paths: impl IntoIterator<Item = PathBuf>,
    ctx: &mut Ctx<'_, '_>,
) -> Result<Vec<Options>> {
    paths
        .into_iter()
        .map(|path| parse_string_project(&path, ctx))
        .collect::<Result<Vec<_>>>()
        .map(|options| options.into_iter().flatten().collect())
}

pub(super) fn string_project_paths(specifier: &str, ctx: &Ctx<'_, '_>) -> Vec<PathBuf> {
    string_project_paths_with_resolver(specifier, ctx.path, ctx.resolver)
}

pub(in crate::integration_tests::test_config::vitest) fn string_project_paths_with_resolver(
    specifier: &str,
    declaration_path: &Path,
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
) -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    if is_file_like_project_specifier(specifier) {
        if let Some(path) = resolver.resolve(specifier, declaration_path) {
            if is_runtime_project_source(&path) {
                paths.insert(path);
            }
        }
    }
    let Some(visible) = resolver.visible_files() else {
        return paths.into_iter().collect();
    };
    let base = declaration_path.parent().unwrap_or(Path::new("."));
    let visible_specifier = specifier.trim_start_matches("./");
    let pattern = crate::codebase::ts_resolver::normalize_path(&base.join(visible_specifier));
    let has_glob = visible_specifier.contains(['*', '?', '[', '{']);
    let direct_glob = has_glob.then(|| {
        GlobBuilder::new(&slash_path(&pattern))
            .literal_separator(true)
            .build()
            .map(|glob| glob.compile_matcher())
    });
    let folder_glob = has_glob.then(|| visible_folder_config_glob(&slash_path(&pattern)));
    for path in visible {
        let direct_match = match &direct_glob {
            Some(Ok(glob)) => glob.is_match(slash_path(path)),
            Some(Err(_)) => false,
            None => path == &pattern,
        };
        let folder_match = match &folder_glob {
            Some(Ok(glob)) => glob.is_match(slash_path(path)),
            Some(Err(_)) => false,
            None => path.parent() == Some(pattern.as_path()),
        };
        if direct_match || (folder_match && is_vitest_project_config(path)) {
            paths.insert(path.clone());
        }
    }
    paths.into_iter().collect()
}

fn is_file_like_project_specifier(specifier: &str) -> bool {
    let path = Path::new(specifier);
    is_runtime_project_source(path) || crate::integration_tests::is_vitest_project_array_path(path)
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn visible_folder_config_glob(specifier: &str) -> Result<globset::GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    // A folder glob selects concrete project roots. Only config files directly
    // inside each matched root belong to that entry; descendants require a
    // separate explicit folder glob (for example `packages/business/*`).
    builder.add(
        GlobBuilder::new(&format!("{}/*", specifier.trim_end_matches('/')))
            .literal_separator(true)
            .build()?,
    );
    builder.build()
}

fn parse_string_project(path: &Path, ctx: &mut Ctx<'_, '_>) -> Result<Option<Options>> {
    parse_string_project_with_resolver(path, ctx.resolver, ctx.seen)
}

pub(in crate::integration_tests::test_config::vitest) fn parse_string_project_with_resolver(
    path: &Path,
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<Option<Options>> {
    if !seen.insert(path.to_path_buf()) {
        return Ok(None);
    }
    let result = match crate::integration_tests::runner_config::read_request_source(path) {
        Err(_) => Ok(None),
        Ok(source) if is_runtime_project_source(path) => {
            crate::integration_tests::runner_config::with_program(
                path,
                &source,
                |program, source| {
                    parse_string_project_program(program, source, path, resolver, seen)
                },
            )
            .and_then(|options| options)
        }
        Ok(source) => crate::ast::with_recovered_typescript_program_observed(
            path,
            &source,
            || {},
            |program, source, diagnostic| {
                diagnostic.map_or_else(
                    || parse_string_project_program(program, source, path, resolver, seen),
                    |diagnostic| Err(anyhow::anyhow!(diagnostic)),
                )
            },
        )
        .and_then(|options| options),
    };
    seen.remove(path);
    result
}

fn parse_string_project_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
    path: &Path,
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<Option<Options>> {
    let bindings = shared::top_level_object_bindings(program);
    let Some(object) = shared::default_export_object(program, &bindings) else {
        return Ok(None);
    };
    let mut local_seen = BTreeSet::new();
    let mut object_seen = BTreeSet::new();
    let mut project_ctx = Ctx {
        source,
        bindings,
        functions: top_level_function_bodies(program),
        imports: import_bindings(program),
        resolver,
        path,
        seen,
        local_seen: &mut local_seen,
        object_seen: &mut object_seen,
    };
    let mut options = objects::project_options(object, &mut project_ctx)?;
    options.standalone_config = true;
    options.config_base = path.parent().map(Path::to_path_buf);
    options.standalone_config_path = Some(path.to_path_buf());
    Ok(Some(options))
}

#[cfg(test)]
mod tests;
