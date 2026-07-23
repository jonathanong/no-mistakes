use crate::codebase::ts_resolver::ImportResolution;
use crate::integration_tests::test_config::vitest::project_arrays::imports::import_sources;
use crate::integration_tests::types::VitestSetupDependency;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// A setup module closure is normally shallow, but a malformed import graph
// must not make a config analysis unbounded. Sources and parsed programs are
// request-cached by `runner_config`; this is a hard safety limit, not a cache.
const MAX_RUNTIME_SETUP_MODULES: usize = 64;

pub(crate) fn resolve_setup_dependencies<'a>(
    dependencies: impl Iterator<Item = &'a mut VitestSetupDependency>,
    project_root: &Path,
    resolver: &dyn ImportResolution,
) {
    // `ImportResolver` takes an importing file, while Vitest resolves these
    // fields from the effective project root. A stable synthetic filename
    // makes its parent exactly that root without reading or executing config.
    let resolution_source = project_root.join(".no-mistakes-vitest-setup.ts");
    for dependency in dependencies {
        dependency.resolution_base = project_root.to_path_buf();
        dependency.resolved_path = None;
        dependency.transitive_trigger_paths.clear();
        dependency
            .trigger_paths
            .retain(|path| !dependency.resolver_candidate_paths.contains(path));
        dependency.resolver_candidate_paths.clear();
        if let Some(specifier) = dependency.specifier.as_deref() {
            let candidates = resolver
                .resolution_candidates(specifier, &resolution_source)
                .into_iter()
                .filter(|candidate| is_runtime_module(candidate))
                .collect::<BTreeSet<_>>();
            dependency.trigger_paths.extend(candidates.iter().cloned());
            dependency.resolver_candidate_paths = candidates;
        }
        dependency.resolved_path = dependency.specifier.as_deref().and_then(|specifier| {
            resolver
                .resolve(specifier, &resolution_source)
                .filter(|path| is_runtime_module(path))
        });
        if let Some(path) = dependency.resolved_path.as_ref() {
            runtime_setup_candidates(path, resolver, &mut dependency.transitive_trigger_paths);
        }
    }
}

/// Retain runtime import candidates for resolved setup modules too. A deleted
/// transitive helper no longer has a graph edge, so its canonical candidate is
/// needed to retain the owning project's bounded impact fallback.
fn runtime_setup_candidates(
    path: &Path,
    resolver: &dyn ImportResolution,
    candidates: &mut BTreeSet<PathBuf>,
) {
    let mut seen = BTreeSet::new();
    collect_runtime_setup_candidates(path, resolver, candidates, &mut seen);
}

fn collect_runtime_setup_candidates(
    path: &Path,
    resolver: &dyn ImportResolution,
    candidates: &mut BTreeSet<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
) {
    if seen.len() >= MAX_RUNTIME_SETUP_MODULES || !seen.insert(path.to_path_buf()) {
        return;
    }
    let Ok(source) = crate::integration_tests::runner_config::read_request_source(path) else {
        return;
    };
    let _ = crate::integration_tests::runner_config::with_program(path, &source, |program, _| {
        for specifier in import_sources(program) {
            candidates.extend(
                resolver
                    .resolution_candidates(&specifier, path)
                    .into_iter()
                    .filter(|candidate| is_runtime_module(candidate)),
            );
            if let Some(dependency) = resolver
                .resolve(&specifier, path)
                .filter(|path| is_runtime_module(path))
            {
                collect_runtime_setup_candidates(&dependency, resolver, candidates, seen);
            }
        }
        for import in crate::codebase::dependencies::extract::extract_imports_from_program(program)
            .into_iter()
            .filter(|import| {
                matches!(
                    import.kind,
                    crate::codebase::dependencies::extract::ImportKind::Dynamic
                )
            })
        {
            candidates.extend(
                resolver
                    .resolution_candidates(&import.specifier, path)
                    .into_iter()
                    .filter(|candidate| is_runtime_module(candidate)),
            );
            if let Some(dependency) = resolver
                .resolve(&import.specifier, path)
                .filter(|path| is_runtime_module(path))
            {
                collect_runtime_setup_candidates(&dependency, resolver, candidates, seen);
            }
        }
    });
}

pub(crate) fn is_runtime_module(path: &Path) -> bool {
    !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
        })
}

#[cfg(test)]
#[path = "setup_resolution/tests.rs"]
mod tests;
