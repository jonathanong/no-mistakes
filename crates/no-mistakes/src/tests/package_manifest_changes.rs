use super::plan::relative_path;
use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::ts_source::SourceStore;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod classify;
use classify::{classify_change, PackageManifestChange};
mod scope;

#[derive(Clone, Default)]
pub(crate) struct PackageManifestAnalysis {
    /// Dependency names paired with the changed manifest that declared them.
    /// A package name alone is not a sound seed in a monorepo: two workspaces
    /// can both depend on it while only one declaration changed.
    pub(crate) changed_packages: Vec<(String, String, PathBuf)>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) fallback_triggered: bool,
    broad_trigger_exclusions: HashSet<PathBuf>,
    handled_files: HashSet<PathBuf>,
}

impl PackageManifestAnalysis {
    pub(crate) fn excludes_broad_trigger(&self, path: &Path) -> bool {
        self.broad_trigger_exclusions.contains(path)
    }

    pub(crate) fn handles(&self, path: &Path) -> bool {
        self.handled_files.contains(path)
    }
}

pub(crate) fn analyze_package_manifest_changes(
    root: &Path,
    changed_files: &[PathBuf],
    revisions: &RevisionSources,
    workspace_map: &no_mistakes::codebase::workspaces::WorkspaceMap,
    sources: &SourceStore,
) -> PackageManifestAnalysis {
    let mut analysis = PackageManifestAnalysis::default();
    let workspace_scope = scope::WorkspaceManifestScope::prepare(root, workspace_map, sources);
    for manifest in changed_files
        .iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) == Some("package.json"))
        .filter(|path| workspace_scope.contains(path))
    {
        let file = relative_path(root, manifest);
        let Some(base) = revisions.base_name() else {
            diagnose(
                &mut analysis,
                manifest,
                &file,
                "package-manifest-no-baseline",
                format!("Could not determine old content of `{file}`. Provide `--base` to enable targeted package dependency analysis; full-suite selection requires global fallback opt-in."),
            );
            continue;
        };
        let Some(before) = revisions.read_base(manifest) else {
            diagnose(
                &mut analysis,
                manifest,
                &file,
                "package-manifest-no-baseline",
                format!("Could not read `{file}` at base ref `{base}`; full-suite selection requires global fallback opt-in"),
            );
            continue;
        };
        let after = revisions.read_after(manifest);
        let Some(after) = after else {
            let detail = if revisions.is_diff_only() {
                "Could not determine new content in diff-only mode. Provide `--head` to enable targeted package dependency analysis"
            } else {
                "Could not read the requested head"
            };
            diagnose(
                &mut analysis,
                manifest,
                &file,
                "package-manifest-no-baseline",
                format!(
                    "{detail} for `{file}`; full-suite selection requires global fallback opt-in"
                ),
            );
            continue;
        };
        match classify_change(&before, &after) {
            Ok(PackageManifestChange::DependencyOnly(packages)) => {
                analysis.broad_trigger_exclusions.insert(manifest.clone());
                analysis.handled_files.insert(manifest.clone());
                analysis.changed_packages.extend(
                    packages
                        .into_iter()
                        .map(|package| (package, file.clone(), manifest.clone())),
                );
            }
            Ok(PackageManifestChange::FormattingOnly) => {
                analysis.broad_trigger_exclusions.insert(manifest.clone());
                analysis.handled_files.insert(manifest.clone());
            }
            Ok(PackageManifestChange::Broad) => {}
            Err(()) => diagnose(
                &mut analysis,
                manifest,
                &file,
                "package-manifest-malformed",
                format!("`{file}` is malformed at a compared revision; full-suite selection requires global fallback opt-in"),
            ),
        }
    }
    analysis
}

fn diagnose(
    analysis: &mut PackageManifestAnalysis,
    manifest: &Path,
    file: &str,
    r#type: &str,
    message: String,
) {
    analysis
        .broad_trigger_exclusions
        .insert(manifest.to_path_buf());
    analysis.handled_files.insert(manifest.to_path_buf());
    analysis.warnings.push(Warning {
        r#type: r#type.to_string(),
        message,
        file: file.to_string(),
        line: None,
    });
    analysis.fallback_triggered = true;
}

#[cfg(test)]
#[path = "package_manifest_changes/workspace_scope_tests.rs"]
mod workspace_scope_tests;

#[cfg(test)]
#[path = "package_manifest_changes/pnpm_installation_sections_tests.rs"]
mod pnpm_installation_sections_tests;

#[cfg(test)]
#[path = "package_manifest_changes/pnpm_traceability_tests.rs"]
mod pnpm_traceability_tests;

#[cfg(test)]
#[path = "package_manifest_changes/dependency_baseline_incomplete_tests.rs"]
mod dependency_baseline_incomplete_tests;

#[cfg(test)]
#[path = "package_manifest_changes_tests.rs"]
mod tests;
