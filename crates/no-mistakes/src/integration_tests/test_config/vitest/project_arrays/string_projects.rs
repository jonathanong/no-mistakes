use super::{import_bindings, objects, shared, top_level_function_bodies, Ctx, Options};
use anyhow::Result;
use globset::{Glob, GlobSetBuilder};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

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
    let mut paths = BTreeSet::new();
    if let Some(path) = ctx.resolver.resolve(specifier, ctx.path) {
        paths.insert(path);
    }
    let Some(visible) = ctx.resolver.visible_files() else {
        return paths.into_iter().collect();
    };
    let base = ctx.path.parent().unwrap_or_else(|| Path::new("."));
    let visible_specifier = specifier.trim_start_matches("./");
    let has_glob = visible_specifier.contains(['*', '?', '[', '{']);
    let glob = has_glob.then(|| {
        visible_config_glob(&slash_path(&crate::codebase::ts_resolver::normalize_path(
            &base.join(visible_specifier),
        )))
    });
    for path in visible.iter().filter(|path| is_vitest_project_config(path)) {
        let matches = match &glob {
            Some(Ok(glob)) => glob.is_match(slash_path(path)),
            Some(Err(_)) => false,
            None => path.starts_with(crate::codebase::ts_resolver::normalize_path(
                &base.join(visible_specifier),
            )),
        };
        if matches {
            paths.insert(path.clone());
        }
    }
    paths.into_iter().collect()
}

fn slash_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn visible_config_glob(specifier: &str) -> Result<globset::GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new(specifier)?);
    builder.add(Glob::new(&format!(
        "{}/**",
        specifier.trim_end_matches('/')
    ))?);
    builder.build()
}

fn is_vitest_project_config(path: &Path) -> bool {
    // Keep config discovery aligned with the resolver's executable TS/JS
    // extensions. Declaration files are intentionally not Vitest configs.
    const EXTENSIONS: &[&str] = &["mts", "ts", "mjs", "js", "cjs", "cts"];
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
    {
        return false;
    }
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    if !EXTENSIONS.contains(&extension) {
        return false;
    }
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .unwrap_or_default();
    stem == "vitest.workspace"
        || stem == "vitest.projects"
        || stem == "vitest.config"
        || stem.starts_with("vitest.config.")
        || stem == "vite.config"
        || stem.starts_with("vite.config.")
        || named_config_stem(stem.as_ref(), "vitest")
        || named_config_stem(stem.as_ref(), "vite")
}

fn named_config_stem(stem: &str, runner: &str) -> bool {
    let Some(name) = stem
        .strip_prefix(runner)
        .and_then(|stem| stem.strip_prefix('.'))
        .and_then(|stem| stem.strip_suffix(".config"))
    else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn parse_string_project(path: &Path, ctx: &mut Ctx<'_, '_>) -> Result<Option<Options>> {
    if !ctx.seen.insert(path.to_path_buf()) {
        return Ok(None);
    }
    let result = match crate::integration_tests::runner_config::read_request_source(path) {
        Err(_) => Ok(None),
        Ok(source) => crate::integration_tests::runner_config::with_program(
            path,
            &source,
            |program, source| {
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
                    resolver: ctx.resolver,
                    path,
                    seen: ctx.seen,
                    local_seen: &mut local_seen,
                    object_seen: &mut object_seen,
                };
                let mut options = objects::project_options(object, &mut project_ctx)?;
                options.standalone_config = true;
                options.config_base = path.parent().map(Path::to_path_buf);
                options.standalone_config_path = Some(path.to_path_buf());
                Ok(Some(options))
            },
        )
        .and_then(|options| options),
    };
    ctx.seen.remove(path);
    result
}

#[cfg(test)]
mod tests;
