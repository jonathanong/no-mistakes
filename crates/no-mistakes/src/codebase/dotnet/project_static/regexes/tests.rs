use super::{static_item_regex, xml_tag, xml_tag_last, xml_tag_regex};

#[test]
fn hoisted_xml_tag_patterns_match_fresh_regexes() {
    let samples = [
        "<AssemblyName>App</AssemblyName>",
        "<assemblyname>  App.Name </assemblyname>",
        "<AssemblyName>\n  Demo\n</AssemblyName>",
        "<RootNamespace>App.Root</RootNamespace>",
        "<IsTestProject>true</IsTestProject>",
        "<IsTestProject> TRUE </IsTestProject>",
        "<EnableDefaultItems>false</EnableDefaultItems><EnableDefaultItems>true</EnableDefaultItems>",
        "<EnableDefaultCompileItems>false</EnableDefaultCompileItems>",
        "<EnableDefaultItems></EnableDefaultItems>",
        "<EnableDefaultItems> </EnableDefaultItems>",
        "<AssemblyName attr=\"x\">Nope</AssemblyName><AssemblyName>Yes</AssemblyName>",
        "<AssemblyNameExtra>No</AssemblyNameExtra><AssemblyName>Yes</AssemblyName>",
        "no tags",
    ];
    for source in samples {
        for tag in super::XML_TAGS {
            let fresh = regex::Regex::new(&format!(r"(?is)<{tag}>\s*([^<]+?)\s*</{tag}>"))
                .expect("oracle xml tag regex");
            let first = fresh.captures(source).and_then(|capture| {
                capture
                    .get(1)
                    .map(|value| value.as_str().trim().to_string())
            });
            let last = fresh.captures_iter(source).last().and_then(|capture| {
                capture
                    .get(1)
                    .map(|value| value.as_str().trim().to_string())
            });
            assert_eq!(xml_tag(source, tag), first, "{tag} first in {source}");
            assert_eq!(xml_tag_last(source, tag), last, "{tag} last in {source}");
        }
    }
    assert!(std::ptr::eq(
        xml_tag_regex("AssemblyName"),
        xml_tag_regex("AssemblyName")
    ));
    assert!(std::ptr::eq(
        xml_tag_regex("EnableDefaultCompileItems"),
        xml_tag_regex("EnableDefaultCompileItems")
    ));
}

#[test]
fn hoisted_static_item_patterns_match_fresh_regexes() {
    let samples = [
        r#"<Project Sdk="Microsoft.NET.Sdk">"#,
        "<project sdk='Other'>",
        r#"<Sdk Name="Microsoft.NET.Sdk" />"#,
        r#"<Import Sdk="Microsoft.NET.Sdk.Web" />"#,
        r#"<PackageReference Include="xunit" Version="2" />"#,
        r#"<packagereference include="A;B" />"#,
        r#"<ProjectReference Include="..\A\A.csproj" />"#,
        r#"<Compile Include="A.cs" />"#,
        r#"<PackageReferenceExtra Include="nope" /><PackageReference Include="yes" />"#,
    ];
    for source in samples {
        for (element, attribute) in super::STATIC_ITEM_PAIRS {
            let fresh = regex::Regex::new(&format!(
                r#"(?is)<{element}\b[^>]*\b{attribute}\s*=\s*["']([^"']+)["']"#
            ))
            .expect("oracle static item regex");
            let expected = fresh
                .captures_iter(source)
                .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
                .collect::<Vec<_>>();
            let actual = static_item_regex(element, attribute)
                .captures_iter(source)
                .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{element} {attribute} in {source}");
        }
    }
    assert!(std::ptr::eq(
        static_item_regex("Project", "Sdk"),
        static_item_regex("Project", "Sdk")
    ));
    assert!(std::ptr::eq(
        static_item_regex("ProjectReference", "Include"),
        static_item_regex("ProjectReference", "Include")
    ));
}
