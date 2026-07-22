use crate::codebase::ts_resolver::ImportResolution;
use crate::integration_tests::types::VitestSetupDependency;
use std::path::Path;

pub(super) fn resolve_setup_dependencies<'a>(
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
        if let Some(specifier) = dependency.specifier.as_deref() {
            dependency
                .trigger_paths
                .extend(resolver.resolution_candidates(specifier, &resolution_source));
        }
        dependency.resolved_path = dependency
            .specifier
            .as_deref()
            .and_then(|specifier| resolver.resolve(specifier, &resolution_source));
    }
}
