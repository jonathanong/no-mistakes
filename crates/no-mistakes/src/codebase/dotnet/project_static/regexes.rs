use regex::Regex;
use std::sync::OnceLock;

pub(super) fn xml_comment_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?s)<!--.*?-->").expect("valid XML comment regex"))
}

pub(super) fn static_compile_operation_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(?is)<Compile\b[^>]*?\b(Include|Remove)\s*=\s*["']([^"']+)["'][^>]*>"#)
            .expect("valid static Compile operation regex")
    })
}

pub(super) fn msbuild_condition_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r#"(?is)\bCondition\s*="#).expect("valid MSBuild condition regex"))
}

pub(super) fn conditional_property_group_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?is)<PropertyGroup\b[^>]*\bCondition\s*=\s*["'][^"']*["'][^>]*>.*?</PropertyGroup\s*>"#,
        )
        .expect("valid conditional PropertyGroup regex")
    })
}

pub(super) fn conditional_item_group_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?is)<ItemGroup\b[^>]*\bCondition\s*=\s*["'][^"']*["'][^>]*>.*?</ItemGroup\s*>"#,
        )
        .expect("valid conditional ItemGroup regex")
    })
}

const XML_TAGS: &[&str] = &[
    "AssemblyName",
    "RootNamespace",
    "IsTestProject",
    "EnableDefaultItems",
    "EnableDefaultCompileItems",
];

const STATIC_ITEM_PAIRS: &[(&str, &str)] = &[
    ("Project", "Sdk"),
    ("Sdk", "Name"),
    ("Import", "Sdk"),
    ("PackageReference", "Include"),
    ("ProjectReference", "Include"),
];

pub(super) fn xml_tag(source: &str, tag: &str) -> Option<String> {
    xml_tag_regex(tag)
        .captures(source)
        .and_then(|capture| capture.get(1))
        .map(|value| value.as_str().trim().to_string())
}

pub(super) fn xml_tag_last(source: &str, tag: &str) -> Option<String> {
    xml_tag_regex(tag)
        .captures_iter(source)
        .last()
        .and_then(|capture| capture.get(1))
        .map(|value| value.as_str().trim().to_string())
}

fn xml_tag_regex(tag: &str) -> &'static Regex {
    static TAGS: OnceLock<[Regex; 5]> = OnceLock::new();
    let tags: &'static [Regex; 5] = TAGS.get_or_init(|| {
        std::array::from_fn(|index| {
            let tag = XML_TAGS[index];
            Regex::new(&format!(r"(?is)<{tag}>\s*([^<]+?)\s*</{tag}>"))
                .expect("valid xml tag regex")
        })
    });
    let index = XML_TAGS
        .iter()
        .position(|name| *name == tag)
        .unwrap_or_else(|| panic!("xml tag regex is not precompiled: {tag}"));
    &tags[index]
}

pub(super) fn static_item_regex(element: &str, attribute: &str) -> &'static Regex {
    static ITEMS: OnceLock<[Regex; 5]> = OnceLock::new();
    let items: &'static [Regex; 5] = ITEMS.get_or_init(|| {
        std::array::from_fn(|index| {
            let (element, attribute) = STATIC_ITEM_PAIRS[index];
            Regex::new(&format!(
                r#"(?is)<{element}\b[^>]*\b{attribute}\s*=\s*["']([^"']+)["']"#
            ))
            .expect("valid static MSBuild item regex")
        })
    });
    let index = STATIC_ITEM_PAIRS
        .iter()
        .position(|(element_name, attribute_name)| {
            *element_name == element && *attribute_name == attribute
        })
        .unwrap_or_else(|| {
            panic!("static MSBuild item regex is not precompiled: {element} {attribute}")
        });
    &items[index]
}

#[cfg(test)]
mod tests;
