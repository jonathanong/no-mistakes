use super::{
    dependency_only_central_packages_change, dependency_only_lockfile_change,
    dependency_only_project_change, fixture,
};
use std::collections::BTreeSet;

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
                .unwrap_or_else(|error| panic!("{change}: {error:?}"))
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
    assert!(project_formatting.formatting_only);
    assert!(project_formatting.changed_dependencies.is_empty());
    let trailing_whitespace =
        dependency_only_project_change(&project, &format!("{project}\n")).unwrap();
    assert!(!trailing_whitespace.dependency_only);
    assert!(trailing_whitespace.formatting_only);
    let central_formatting =
        dependency_only_central_packages_change(&central, &fixture("central-formatting.props"))
            .unwrap();
    assert!(!central_formatting.dependency_only);
    assert!(central_formatting.formatting_only);
    assert!(central_formatting.changed_dependencies.is_empty());
    let framework_change =
        dependency_only_project_change(&project, &fixture("target-framework.csproj")).unwrap();
    assert!(!framework_change.dependency_only);
    assert!(!framework_change.formatting_only);
    assert!(framework_change.changed_dependencies.is_empty());
    let reordered_metadata = dependency_only_project_change(
        &fixture("metadata-base.csproj"),
        &fixture("metadata-reordered.csproj"),
    )
    .unwrap();
    assert!(!reordered_metadata.dependency_only);
    assert!(!reordered_metadata.formatting_only);
    assert!(reordered_metadata.changed_dependencies.is_empty());
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
    let formatting = dependency_only_lockfile_change(
        &fixture("packages.lock.json"),
        &fixture("lock-formatting.json"),
    )
    .unwrap();
    assert!(!formatting.dependency_only);
    assert!(formatting.formatting_only);
    let root_version = dependency_only_lockfile_change(
        &fixture("packages.lock.json"),
        &fixture("lock-root-version.json"),
    )
    .unwrap();
    assert!(!root_version.dependency_only);
    assert!(!root_version.formatting_only);
    assert!(root_version.changed_dependencies.is_empty());
    let mixed_root_and_dependency = dependency_only_lockfile_change(
        &fixture("packages.lock.json"),
        &fixture("lock-root-version-and-dependency.json"),
    )
    .unwrap();
    assert!(!mixed_root_and_dependency.dependency_only);
    assert!(!mixed_root_and_dependency.formatting_only);
    assert_eq!(
        mixed_root_and_dependency.changed_dependencies,
        BTreeSet::from(["net9.0:Alpha".to_string()])
    );
}
