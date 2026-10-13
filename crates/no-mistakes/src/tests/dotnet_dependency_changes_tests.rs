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
    for (change, target) in [
        ("formatting.csproj", "base.csproj"),
        ("central-formatting.props", "Directory.Packages.props"),
        ("lock-formatting.json", "packages.lock.json"),
    ] {
        let formatting = analysis(change, target);
        assert!(formatting.artifacts.is_empty(), "{target}");
        assert!(formatting.warnings.is_empty(), "{target}");
        assert_eq!(formatting.handled.len(), 1, "{target}");
        assert!(
            formatting.handled.iter().any(|path| path.ends_with(target)),
            "{target}"
        );
    }
    for (change, target) in [
        ("target-framework.csproj", "base.csproj"),
        ("lock-root-version.json", "packages.lock.json"),
    ] {
        let semantic = analysis(change, target);
        assert!(semantic.artifacts.is_empty(), "{target}");
        assert!(semantic.warnings.is_empty(), "{target}");
        assert!(semantic.handled.is_empty(), "{target}");
    }
    let mixed_lock = analysis(
        "lock-root-version-and-dependency.json",
        "packages.lock.json",
    );
    assert!(mixed_lock.artifacts.is_empty());
    assert_eq!(
        mixed_lock.warnings[0].r#type,
        "dotnet-dependency-mixed-configuration"
    );
    assert!(mixed_lock.handled.is_empty());
}

#[test]
fn diagnostic_and_baseline_cases_are_handled_with_typed_warning() {
    let fixture = fixture();
    let root = fixture.path().canonicalize().unwrap();
    let path = root.join("base.csproj");
    let no_base_args = args(&root, path.clone(), None);
    let no_base_revisions = revisions(&root, &no_base_args);
    let no_base =
        analyze_dotnet_dependency_changes(&root, std::slice::from_ref(&path), &no_base_revisions);
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

#[test]
fn recognizes_rid_lockfiles_as_semantic_dependency_artifacts() {
    let lock = analysis(
        "lock-version.json",
        "packages.net10.0-maccatalyst.arm64.lock.json",
    );
    assert_eq!(lock.artifacts.len(), 1);
    assert_eq!(lock.artifacts[0].kind, DotnetArtifactKind::Lockfile);
    assert_eq!(
        lock.artifacts[0].changed_dependencies,
        BTreeSet::from(["net9.0:Alpha".to_string()])
    );
}
