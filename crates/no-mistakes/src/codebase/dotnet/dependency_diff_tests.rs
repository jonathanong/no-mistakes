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

#[test]
fn open_tag_parser_handles_self_closing_attributes_and_malformed_forms() {
    assert_eq!(
        crate::codebase::dotnet::dependency_diff::parse_open_tag(" Item Include = 'Alpha' /"),
        Ok((
            "Item".to_string(),
            vec![("Include".to_string(), "Alpha".to_string())],
            true,
        ))
    );
    for tag in [
        "",
        "/ nope",
        "Item Include",
        "Item Include=bare",
        "Item Include='open",
    ] {
        assert_eq!(
            crate::codebase::dotnet::dependency_diff::parse_open_tag(tag),
            Err(DotnetDependencyDiagnostic::MalformedXml),
            "{tag}"
        );
    }
}

#[test]
fn dependency_fingerprint_handles_comments_text_nesting_and_malformed_xml() {
    let fingerprint = crate::codebase::dotnet::dependency_fingerprint::dependency_fingerprint(
        "<Project><!-- comment --><ItemGroup>text<PackageReference Version=\"1\" Include=\"Alpha\"/></ItemGroup></Project>",
    )
    .unwrap();
    assert!(!fingerprint.contains("comment"));
    assert!(fingerprint.contains("text"));
    assert_eq!(
        crate::codebase::dotnet::dependency_fingerprint::dependency_fingerprint(
            "<Project><Item /></Project>"
        ),
        crate::codebase::dotnet::dependency_fingerprint::dependency_fingerprint(
            "<Project><Item/></Project>"
        )
    );
    for source in ["<Project>", "</Project>", "<Project></Item>", "text"] {
        assert_eq!(
            crate::codebase::dotnet::dependency_fingerprint::dependency_fingerprint(source),
            Err(DotnetDependencyDiagnostic::MalformedXml),
            "{source}"
        );
    }
}

#[test]
fn project_xml_supports_trivia_but_rejects_invalid_document_structure() {
    let before = r#"<?xml version="1.0"?><Project><!-- note --><![CDATA[ignored]]><ItemGroup><PackageReference Include="Alpha" Version="1.0" /></ItemGroup></Project>"#;
    let after = r#"<?xml version="1.0"?><Project><!-- note --><![CDATA[ignored]]><ItemGroup><PackageReference Include="Alpha" Version="2.0" /></ItemGroup></Project>"#;
    assert!(
        dependency_only_project_change(before, after)
            .unwrap()
            .dependency_only
    );

    for source in [
        "<Assembly />",
        "text<Project />",
        "<Project><![CDATA[unterminated</Project>",
    ] {
        assert_eq!(
            dependency_only_project_change(before, source),
            Err(DotnetDependencyDiagnostic::MalformedXml),
            "{source}"
        );
    }
}
