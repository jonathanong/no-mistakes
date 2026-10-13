use super::plan::relative_path;
use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::swift::{
    dependency_only_manifest_change, formatting_only_manifest_change, SwiftManifestDiagnostic,
};
use no_mistakes::config::v2::NoMistakesConfig;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Default)]
pub(crate) struct SwiftManifestAnalysis {
    pub(crate) seeds: Vec<PathBuf>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) fallback_triggered: bool,
    handled_files: HashSet<PathBuf>,
    dependency_only_files: HashSet<PathBuf>,
}
impl SwiftManifestAnalysis {
    pub(crate) fn handles(&self, path: &Path) -> bool {
        self.handled_files.contains(path)
    }
    pub(crate) fn dependency_only_files(&self) -> &HashSet<PathBuf> {
        &self.dependency_only_files
    }
}

pub(crate) fn analyze_swift_manifest_changes(
    root: &Path,
    config: &NoMistakesConfig,
    changed: &[PathBuf],
    revisions: &RevisionSources,
) -> SwiftManifestAnalysis {
    let mut analysis = SwiftManifestAnalysis::default();
    for package in &config.tests.swift.packages {
        let manifest = root
            .join(package.trim_end_matches('/'))
            .join("Package.swift");
        if !changed.contains(&manifest) {
            continue;
        }
        let file = relative_path(root, &manifest);
        let Some(base) = revisions.base_name() else {
            analysis.handled_files.insert(manifest.clone());
            warning(&mut analysis, "swift-manifest-no-baseline", format!("Could not determine old content of `{file}`. Provide `--base` to enable targeted Swift dependency analysis; full-suite selection requires global fallback opt-in."), file);
            continue;
        };
        let Some(before) = revisions.read_base(&manifest) else {
            analysis.handled_files.insert(manifest.clone());
            warning(&mut analysis, "swift-manifest-no-baseline", format!("Could not read `{file}` at base ref `{base}`; full-suite selection requires global fallback opt-in"), file);
            continue;
        };
        if revisions.is_diff_only() {
            analysis.handled_files.insert(manifest);
            warning(
                &mut analysis,
                "swift-manifest-no-baseline",
                format!(
                    "Could not determine new content of `{file}` in diff-only mode. Provide `--head` to enable targeted Swift dependency analysis; full-suite selection requires global fallback opt-in."
                ),
                file,
            );
            continue;
        }
        let after = revisions.read_after_or_empty(&manifest);
        if formatting_only_manifest_change(&before, &after) {
            analysis.handled_files.insert(manifest.clone());
            analysis.dependency_only_files.insert(manifest);
            continue;
        }
        match dependency_only_manifest_change(&before, &after) {
            Ok(true) => {
                analysis.handled_files.insert(manifest.clone());
                analysis.dependency_only_files.insert(manifest.clone());
                analysis.seeds.push(manifest);
            }
            Ok(false) => {}
            Err(SwiftManifestDiagnostic::UnsupportedDynamicDeclaration) => {
                analysis.handled_files.insert(manifest);
                warning(&mut analysis, "swift-manifest-unsupported-dynamic", format!("`{file}` has an unsupported dynamic dependency declaration; full-suite selection requires global fallback opt-in"), file)
            }
        }
    }
    analysis
}
fn warning(analysis: &mut SwiftManifestAnalysis, r#type: &str, message: String, file: String) {
    analysis.warnings.push(Warning {
        r#type: r#type.to_string(),
        message,
        file,
        line: None,
    });
    analysis.fallback_triggered = true;
}

#[cfg(test)]
#[path = "swift_manifest_changes_tests.rs"]
mod tests;
