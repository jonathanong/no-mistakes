use super::*;
use no_mistakes::codebase::dotnet::{DotnetFactMap, DotnetProjectFacts};
use std::collections::BTreeSet;

#[test]
fn central_consumers_are_scoped_to_nearest_props() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/test-plan/dotnet-semantic-seeds/fixture");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let root = fixture.path().canonicalize().unwrap();
    let mut facts = DotnetFactMap::default();
    for (name, package) in [("core", "Core.Only"), ("app", "App.Only")] {
        let dir = root.join(name);
        let path = dir.join(format!("{name}.csproj"));
        facts.projects.insert(
            path.clone(),
            DotnetProjectFacts {
                project_path: path,
                project_dir: dir,
                package_references: BTreeSet::from([package.to_string()]),
                ..Default::default()
            },
        );
    }
    let central_files = [
        root.join("Directory.Packages.props"),
        root.join("app/Directory.Packages.props"),
    ];
    assert_eq!(
        central_package_seed_roots(
            &root,
            &root.join("app/Directory.Packages.props"),
            &BTreeSet::from(["App.Only".to_string()]),
            &facts,
            &central_files,
            None,
        ),
        vec![root.join("app/app.csproj")]
    );
    assert_eq!(
        central_package_seed_roots(
            &root,
            &root.join("Directory.Packages.props"),
            &BTreeSet::from(["Core.Only".to_string()]),
            &facts,
            &central_files,
            None,
        ),
        vec![root.join("core/core.csproj")]
    );
    assert!(central_package_seed_roots(
        &root,
        &root.join("Directory.Packages.props"),
        &BTreeSet::from(["App.Only".to_string()]),
        &facts,
        &central_files,
        None,
    )
    .is_empty());
    assert_eq!(
        central_package_seed_roots(
            &root,
            &root.join("app/Directory.Packages.props"),
            &BTreeSet::from(["app.only".to_string()]),
            &facts,
            &central_files,
            None,
        ),
        vec![root.join("app/app.csproj")],
        "NuGet package identities are case-insensitive"
    );
    assert!(central_package_seed_roots(
        &root,
        &root.join("Directory.Packages.props"),
        &BTreeSet::from(["Other".to_string()]),
        &facts,
        &central_files,
        None,
    )
    .is_empty());
}

#[test]
fn lockfile_seeds_every_project_that_shares_its_owner_directory() {
    let owner = PathBuf::from("/repo/apps");
    let mut facts = DotnetFactMap::default();
    for name in ["First", "Second"] {
        let project_path = owner.join(format!("{name}.csproj"));
        facts.projects.insert(
            project_path.clone(),
            DotnetProjectFacts {
                project_path,
                project_dir: owner.clone(),
                ..Default::default()
            },
        );
    }
    let artifact = crate::tests::dotnet_dependency_changes::DotnetDependencyArtifact {
        path: owner.join("packages.lock.json"),
        kind: crate::tests::dotnet_dependency_changes::DotnetArtifactKind::Lockfile,
        changed_dependencies: BTreeSet::new(),
        owning_project: Some(owner),
    };

    assert_eq!(
        lockfile_seed_roots(&artifact, &facts),
        [
            PathBuf::from("/repo/apps/First.csproj"),
            PathBuf::from("/repo/apps/Second.csproj"),
        ]
    );
}
