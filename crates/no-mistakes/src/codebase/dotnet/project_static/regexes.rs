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
