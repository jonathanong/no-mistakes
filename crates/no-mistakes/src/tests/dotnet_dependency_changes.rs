use super::plan::relative_path;
use super::prepared_plan::revisions::RevisionSources;
use super::Warning;
use no_mistakes::codebase::dotnet::{
    dependency_only_central_packages_change, dependency_only_lockfile_change,
    dependency_only_project_change, DotnetDependencyDiagnostic,
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
            Some("packages.lock.json") => DotnetArtifactKind::Lockfile,
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
            Ok(diff) if diff.dependency_only => {
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
            }
            Ok(diff) if !diff.changed_dependencies.is_empty() => {
                push_warning(&mut analysis, "dotnet-dependency-mixed-configuration", file)
            }
            Ok(_) => {}
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
mod tests {
    use super::*;
    use crate::test_support::{git_commit_all, git_init, materialize_saved_fixture};
    use crate::tests::{PlanArgs, TestFramework};

    fn args(root: &Path, changed: PathBuf, base: Option<&str>) -> PlanArgs {
        PlanArgs {
            framework: Some(TestFramework::Dotnet),
            root: root.to_path_buf(),
            config: None,
            tsconfig: None,
            base: base.map(str::to_string),
            head: None,
            from_git_diff: None,
            changed_file: vec![changed],
            changed_files: None,
            diff: None,
            diff_stdin: false,
            diff_command: None,
            entrypoints: Vec::new(),
            entrypoint_symbols: Vec::new(),
            include_symbols: false,
            diff_content: None,
            environment: "pre-push".to_string(),
            limit_percent: None,
            limit_files: None,
            global_config_fallback: Some(false),
            direct_test_owner: false,
            format: None,
            json: false,
            include_comment: false,
            include_glob: Vec::new(),
        }
    }
    fn fixture() -> tempfile::TempDir {
        materialize_saved_fixture(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/test-plan/dotnet-dependency-diff"),
        )
    }
    fn revisions(root: &Path, args: &PlanArgs) -> RevisionSources {
        let snapshot = no_mistakes::codebase::ts_source::VisiblePathSnapshot::new(root);
        RevisionSources::prepare(root, args, snapshot.source_store_for(root))
    }
    fn analysis(change: &str, target: &str) -> DotnetDependencyAnalysis {
        let fixture = fixture();
        let root = fixture.path().canonicalize().unwrap();
        git_init(&root);
        git_commit_all(&root, "base");
        std::fs::copy(root.join(change), root.join(target)).unwrap();
        let args = args(&root, root.join(target), Some("HEAD"));
        let revisions = revisions(&root, &args);
        analyze_dotnet_dependency_changes(&root, &[root.join(target)], &revisions)
    }

    #[test]
    fn records_static_artifacts_and_diagnoses_mixed_configuration() {
        let project = analysis("project-add.csproj", "base.csproj");
        assert_eq!(
            project.artifacts[0].changed_dependencies,
            BTreeSet::from(["Beta".to_string()])
        );
        assert_eq!(
            project.artifacts[0].owning_project,
            Some(project.artifacts[0].path.clone())
        );
        let central = analysis("central-version.props", "Directory.Packages.props");
        assert_eq!(
            central.artifacts[0].changed_dependencies,
            BTreeSet::from(["App.Only".to_string()])
        );
        assert_eq!(central.artifacts[0].owning_project, None);
        let lock = analysis("lock-version.json", "packages.lock.json");
        assert_eq!(
            lock.artifacts[0].changed_dependencies,
            BTreeSet::from(["net9.0:Alpha".to_string()])
        );
        assert_eq!(
            lock.artifacts[0].owning_project,
            lock.artifacts[0].path.parent().map(Path::to_path_buf)
        );
        let mixed = analysis("mixed.csproj", "base.csproj");
        assert!(mixed.handled.is_empty());
        assert!(mixed.artifacts.is_empty());
        assert_eq!(
            mixed.warnings[0].r#type,
            "dotnet-dependency-mixed-configuration"
        );
        let formatting = analysis("formatting.csproj", "base.csproj");
        assert!(formatting.artifacts.is_empty());
    }

    #[test]
    fn diagnostic_and_baseline_cases_are_handled_with_typed_warning() {
        let fixture = fixture();
        let root = fixture.path().canonicalize().unwrap();
        let path = root.join("base.csproj");
        let no_base_args = args(&root, path.clone(), None);
        let no_base_revisions = revisions(&root, &no_base_args);
        let no_base = analyze_dotnet_dependency_changes(
            &root,
            std::slice::from_ref(&path),
            &no_base_revisions,
        );
        assert!(no_base.handles(&path));
        assert_eq!(no_base.warnings[0].r#type, "dotnet-dependency-no-baseline");
        let malformed = analysis("malformed.csproj", "base.csproj");
        assert_eq!(
            malformed.warnings[0].r#type,
            "dotnet-dependency-malformed-xml"
        );
        let unrelated = root.join("readme.md");
        let ignored_args = args(&root, unrelated.clone(), None);
        let ignored_revisions = revisions(&root, &ignored_args);
        let ignored = analyze_dotnet_dependency_changes(&root, &[unrelated], &ignored_revisions);
        assert!(ignored.artifacts.is_empty() && ignored.warnings.is_empty());
    }
}
