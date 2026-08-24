use super::*;
use std::path::PathBuf;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/test-plan/dotnet-dependency-diff")
            .join(name),
    )
    .unwrap()
}

#[test]
fn project_and_central_dependency_deltas_are_semantic() {
    let project = fixture("base.csproj");
    for change in [
        "project-add.csproj",
        "project-remove.csproj",
        "project-version.csproj",
    ] {
        assert!(
            dependency_only_project_change(&project, &fixture(change))
                .unwrap()
                .dependency_only
        );
    }
    let central = fixture("Directory.Packages.props");
    assert!(
        dependency_only_central_packages_change(&central, &fixture("central-version.props"))
            .unwrap()
            .dependency_only
    );
    let project_formatting =
        dependency_only_project_change(&project, &fixture("formatting.csproj")).unwrap();
    assert!(!project_formatting.dependency_only);
    assert!(project_formatting.changed_dependencies.is_empty());
    let central_formatting =
        dependency_only_central_packages_change(&central, &fixture("central-formatting.props"))
            .unwrap();
    assert!(!central_formatting.dependency_only);
    assert!(central_formatting.changed_dependencies.is_empty());
    assert_eq!(
        dependency_only_project_change(&project, &fixture("project-add.csproj"))
            .unwrap()
            .changed_dependencies,
        BTreeSet::from(["Beta".to_string()])
    );
    assert_eq!(
        dependency_only_central_packages_change(&central, &fixture("central-version.props"))
            .unwrap()
            .changed_dependencies,
        BTreeSet::from(["App.Only".to_string()])
    );
}

#[test]
fn lockfile_dependency_records_are_semantic() {
    let diff = dependency_only_lockfile_change(
        &fixture("packages.lock.json"),
        &fixture("lock-version.json"),
    )
    .unwrap();
    assert!(diff.dependency_only);
    assert_eq!(
        diff.changed_dependencies,
        BTreeSet::from(["net9.0:Alpha".to_string()])
    );
    let nested = dependency_only_lockfile_change(
        &fixture("packages.lock.json"),
        &fixture("lock-nested-dependency.json"),
    )
    .unwrap();
    assert!(nested.dependency_only);
    assert_eq!(
        nested.changed_dependencies,
        BTreeSet::from(["net9.0:Alpha".to_string()])
    );
}

#[test]
fn unsafe_or_malformed_inputs_are_diagnostic() {
    let base = fixture("base.csproj");
    assert!(
        !dependency_only_project_change(&base, &fixture("mixed.csproj"))
            .unwrap()
            .dependency_only
    );
    assert!(
        !dependency_only_project_change(
            &fixture("base-exec.csproj"),
            &fixture("mixed-exec-whitespace.csproj"),
        )
        .unwrap()
        .dependency_only,
        "whitespace in an Exec command is semantic, not XML trivia"
    );
    for change in [
        "conditional.csproj",
        "ancestor-static-condition.csproj",
        "condition-whitespace.csproj",
        "imported.csproj",
        "property-expanded.csproj",
        "item-expression.csproj",
        "metadata-expression.csproj",
        "wildcard.csproj",
        "multi-item.csproj",
    ] {
        assert_eq!(
            dependency_only_project_change(&base, &fixture(change)),
            Err(DotnetDependencyDiagnostic::UnsupportedDynamicDeclaration)
        );
    }
    assert_eq!(
        dependency_only_project_change(&base, &fixture("malformed.csproj")),
        Err(DotnetDependencyDiagnostic::MalformedXml)
    );
    assert_eq!(
        dependency_only_project_change(&base, &fixture("mismatched-elements.csproj")),
        Err(DotnetDependencyDiagnostic::MalformedXml)
    );
    assert_eq!(
        dependency_only_lockfile_change(
            &fixture("packages.lock.json"),
            &fixture("malformed-lock.json")
        ),
        Err(DotnetDependencyDiagnostic::MalformedLockfile)
    );
    assert_eq!(
        dependency_only_lockfile_change(
            &fixture("packages.lock.json"),
            &fixture("unsupported-lock.json")
        ),
        Err(DotnetDependencyDiagnostic::UnsupportedLockSchema)
    );
}
