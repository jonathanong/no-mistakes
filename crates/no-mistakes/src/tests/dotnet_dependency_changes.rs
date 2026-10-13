use super::plan::relative_path;
use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::dotnet::{
    dependency_only_central_packages_change, dependency_only_lockfile_change,
    dependency_only_project_change, is_dotnet_lockfile, DotnetDependencyDiagnostic,
};
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DotnetArtifactKind {
    Project,
    CentralPackages,
    Lockfile,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DotnetDependencyArtifact {
    pub(crate) path: PathBuf,
    pub(crate) kind: DotnetArtifactKind,
    pub(crate) changed_dependencies: BTreeSet<String>,
    pub(crate) owning_project: Option<PathBuf>,
}
#[derive(Clone, Default)]
pub(crate) struct DotnetDependencyAnalysis {
    pub(crate) artifacts: Vec<DotnetDependencyArtifact>,
    pub(crate) warnings: Vec<Warning>,
    pub(crate) fallback_triggered: bool,
    handled: HashSet<PathBuf>,
}
impl DotnetDependencyAnalysis {
    pub(crate) fn handles(&self, path: &Path) -> bool {
        self.handled.contains(path)
    }
}

pub(crate) fn analyze_dotnet_dependency_changes(
    root: &Path,
    changed: &[PathBuf],
    revisions: &RevisionSources,
) -> DotnetDependencyAnalysis {
    let mut analysis = DotnetDependencyAnalysis::default();
    for path in changed {
        let kind = match path.file_name().and_then(|x| x.to_str()) {
            Some("Directory.Packages.props") => DotnetArtifactKind::CentralPackages,
            Some(_) if is_dotnet_lockfile(path) => DotnetArtifactKind::Lockfile,
            _ if path.extension().and_then(|x| x.to_str()) == Some("csproj") => {
                DotnetArtifactKind::Project
            }
            _ => continue,
        };
        let file = relative_path(root, path);
        let Some(_) = revisions.base_name() else {
            warning(&mut analysis, path, "dotnet-dependency-no-baseline", file);
            continue;
        };
        let Some(before) = revisions.read_base(path) else {
            warning(&mut analysis, path, "dotnet-dependency-no-baseline", file);
            continue;
        };
        if revisions.is_diff_only() {
            warning(&mut analysis, path, "dotnet-dependency-no-baseline", file);
            continue;
        }
        let after = revisions.read_after_or_empty(path);
        let result = match kind {
            DotnetArtifactKind::Project => dependency_only_project_change(&before, &after),
            DotnetArtifactKind::CentralPackages => {
                dependency_only_central_packages_change(&before, &after)
            }
            DotnetArtifactKind::Lockfile => dependency_only_lockfile_change(&before, &after),
        };
        match result {
            Ok(diff) => {
                if diff.dependency_only {
                    analysis.handled.insert(path.clone());
                    analysis.artifacts.push(DotnetDependencyArtifact {
                        path: path.clone(),
                        kind,
                        changed_dependencies: diff.changed_dependencies,
                        owning_project: match kind {
                            DotnetArtifactKind::Project => Some(path.clone()),
                            DotnetArtifactKind::Lockfile => path.parent().map(Path::to_path_buf),
                            DotnetArtifactKind::CentralPackages => None,
                        },
                    })
                } else if !diff.changed_dependencies.is_empty() {
                    push_warning(&mut analysis, "dotnet-dependency-mixed-configuration", file)
                } else if diff.formatting_only {
                    // The full normalized document is equal, so this edit is
                    // formatting-only. Do not let the raw dependency-file graph edge
                    // select its owning tests.
                    analysis.handled.insert(path.clone());
                }
            }
            Err(d) => warning(&mut analysis, path, diagnostic_type(d), file),
        }
    }
    analysis
}
fn warning(analysis: &mut DotnetDependencyAnalysis, path: &Path, kind: &str, file: String) {
    analysis.handled.insert(path.to_path_buf());
    push_warning(analysis, kind, file);
}

fn push_warning(analysis: &mut DotnetDependencyAnalysis, kind: &str, file: String) {
    analysis.warnings.push(Warning {
        r#type: kind.to_string(),
        message: format!("`{file}` cannot be semantically analyzed"),
        file,
        line: None,
    });
    analysis.fallback_triggered = true;
}
fn diagnostic_type(d: DotnetDependencyDiagnostic) -> &'static str {
    match d {
        DotnetDependencyDiagnostic::UnsupportedDynamicDeclaration => {
            "dotnet-dependency-unsupported-dynamic"
        }
        DotnetDependencyDiagnostic::MalformedXml => "dotnet-dependency-malformed-xml",
        DotnetDependencyDiagnostic::MalformedLockfile => "dotnet-dependency-malformed-lockfile",
        DotnetDependencyDiagnostic::UnsupportedLockSchema => {
            "dotnet-dependency-unsupported-lock-schema"
        }
    }
}

#[cfg(test)]
#[path = "dotnet_dependency_changes_tests.rs"]
mod tests;
