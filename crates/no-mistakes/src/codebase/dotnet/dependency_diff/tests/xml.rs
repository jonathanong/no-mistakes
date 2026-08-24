use super::{dependency_only_project_change, DotnetDependencyDiagnostic};

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
