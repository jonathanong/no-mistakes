use super::{project_options, shared, Ctx, Options};
use crate::integration_tests::test_config::vitest::Extends;
use crate::integration_tests::types::{VitestSetupDependency, VitestSetupField};
use anyhow::Result;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn resolve_config_extends(options: &mut Options, ctx: &mut Ctx<'_, '_>) -> Result<()> {
    let Some(extends) = options.extends.take() else {
        return Ok(());
    };
    let Extends::Config(specifier) = extends else {
        options.extends = Some(extends);
        return Ok(());
    };
    let candidates = ctx.resolver.resolution_candidates(&specifier, ctx.path);
    let Some(path) = ctx.resolver.resolve(&specifier, ctx.path) else {
        add_unresolved_config_extends(options, specifier, candidates, ctx);
        return Ok(());
    };
    if !ctx.seen.insert(path.clone()) {
        return Ok(());
    }
    let inherited = match crate::integration_tests::runner_config::read_request_source(&path) {
        Err(_) => Ok(None),
        Ok(source) => crate::integration_tests::runner_config::with_program(
            &path,
            &source,
            |program, source| {
                let bindings = shared::top_level_object_bindings(program);
                let Some(object) = shared::default_export_object(program, &bindings) else {
                    return Ok(None);
                };
                let mut local_seen = BTreeSet::new();
                let mut object_seen = BTreeSet::new();
                let mut inherited_ctx = Ctx {
                    source,
                    bindings,
                    functions: super::super::top_level_function_bodies(program),
                    imports: super::super::import_bindings(program),
                    resolver: ctx.resolver,
                    path: &path,
                    seen: ctx.seen,
                    local_seen: &mut local_seen,
                    object_seen: &mut object_seen,
                };
                project_options(object, &mut inherited_ctx).map(Some)
            },
        )
        .and_then(|options| options),
    };
    ctx.seen.remove(&path);
    let Some(inherited) = inherited? else {
        add_unresolved_config_extends(options, specifier, candidates, ctx);
        return Ok(());
    };
    add_config_extends_provenance(options, &path, candidates, ctx);
    merge_inherited_options(options, inherited, &path);
    Ok(())
}

fn merge_inherited_options(options: &mut Options, mut inherited: Options, path: &Path) {
    normalize_inherited_root(&mut inherited, path);
    options.name = options.name.take().or(inherited.name);
    options.root = options.root.take().or(inherited.root);
    options.include = options.include.take().or(inherited.include);
    options.exclude = combine_excludes(inherited.exclude, options.exclude.take());
    options.setup_files = crate::integration_tests::test_config::vitest::merge::inherit_setup_files(
        inherited.setup_files,
        options.setup_files.take(),
    );
    options.global_setup =
        crate::integration_tests::test_config::vitest::merge::inherit_setup_files(
            inherited.global_setup,
            options.global_setup.take(),
        );
}

fn normalize_inherited_root(inherited: &mut Options, path: &Path) {
    let Some(root) = inherited.root.as_deref() else {
        return;
    };
    let root = Path::new(root);
    if !root.is_absolute() {
        inherited.root = Some(
            crate::codebase::ts_resolver::normalize_path(
                &path.parent().unwrap_or(Path::new(".")).join(root),
            )
            .to_string_lossy()
            .into_owned(),
        );
    }
}

fn combine_excludes(
    inherited: Option<Vec<String>>,
    local: Option<Vec<String>>,
) -> Option<Vec<String>> {
    let mut excludes = inherited.unwrap_or_default();
    excludes.extend(local.unwrap_or_default());
    (!excludes.is_empty()).then_some(excludes)
}

fn add_unresolved_config_extends(
    options: &mut Options,
    specifier: String,
    mut candidates: BTreeSet<PathBuf>,
    ctx: &Ctx<'_, '_>,
) {
    candidates.insert(ctx.path.to_path_buf());
    let extends_path = Path::new(&specifier);
    if extends_path.is_absolute() || specifier.starts_with('.') {
        let candidate = if extends_path.is_absolute() {
            extends_path.to_path_buf()
        } else {
            ctx.path
                .parent()
                .unwrap_or(Path::new("."))
                .join(extends_path)
        };
        candidates.insert(crate::codebase::ts_resolver::normalize_path(&candidate));
    }
    let declaration_line = ctx
        .source
        .find(&specifier)
        .map(|offset| crate::codebase::ts_source::line_number(ctx.source, offset as u32) as u32)
        .unwrap_or(1);
    let fallback = VitestSetupDependency {
        field: VitestSetupField::SetupFiles,
        specifier: None,
        needs_final_catalog_reparse: false,
        unresolved_config_extends: Some(specifier),
        config_extends_provenance: false,
        resolved_path: None,
        resolution_base: ctx.path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        declaration_path: ctx.path.to_path_buf(),
        declaration_line,
        trigger_paths: candidates,
        resolver_candidate_paths: BTreeSet::new(),
        transitive_trigger_paths: BTreeSet::new(),
    };
    options.setup_files = crate::integration_tests::test_config::vitest::merge::inherit_setup_files(
        Some(vec![fallback]),
        options.setup_files.take(),
    );
}

fn add_config_extends_provenance(
    options: &mut Options,
    path: &Path,
    mut candidates: BTreeSet<PathBuf>,
    ctx: &Ctx<'_, '_>,
) {
    candidates.insert(path.to_path_buf());
    let provenance = VitestSetupDependency {
        field: VitestSetupField::SetupFiles,
        specifier: None,
        needs_final_catalog_reparse: false,
        unresolved_config_extends: None,
        config_extends_provenance: true,
        resolved_path: None,
        resolution_base: ctx.path.parent().unwrap_or(Path::new(".")).to_path_buf(),
        declaration_path: ctx.path.to_path_buf(),
        declaration_line: 1,
        trigger_paths: candidates,
        resolver_candidate_paths: BTreeSet::new(),
        transitive_trigger_paths: BTreeSet::new(),
    };
    options.setup_files = crate::integration_tests::test_config::vitest::merge::inherit_setup_files(
        Some(vec![provenance]),
        options.setup_files.take(),
    );
}
