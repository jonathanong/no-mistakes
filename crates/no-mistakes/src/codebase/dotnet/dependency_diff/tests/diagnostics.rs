use super::{
    dependency_only_lockfile_change, dependency_only_project_change, fixture,
    DotnetDependencyDiagnostic,
};

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
