use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::{
    csharp_http,
    csharp_strip::{strip_comments_and_strings, strip_comments_keep_strings},
    DotnetFileFacts,
};

pub(crate) fn parse_csharp_file_with_sources(
    path: &Path,
    project: Option<PathBuf>,
    sources: Option<&crate::codebase::ts_source::SourceStore>,
) -> Option<DotnetFileFacts> {
    let source = crate::codebase::ts_source::SourceStore::read_optional(sources, path)?;
    let stripped = strip_comments_and_strings(&source);
    Some(DotnetFileFacts {
        path: path.to_path_buf(),
        project,
        namespace: extract_namespace(&stripped),
        usings: extract_usings(&stripped),
        declarations: extract_declarations(&stripped),
        references: extract_references(&stripped),
        has_xunit_tests: has_xunit_tests(&stripped),
        methods: extract_methods(&stripped),
        route_handlers: csharp_http::extract_http_routes(&strip_comments_keep_strings(&source)),
    })
}

fn extract_namespace(source: &str) -> Option<String> {
    csharp_file_namespace_regex()
        .captures(source)
        .or_else(|| csharp_block_namespace_regex().captures(source))
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn extract_usings(source: &str) -> Vec<String> {
    sorted_unique(
        csharp_using_regex()
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    )
}

fn extract_methods(source: &str) -> Vec<String> {
    sorted_unique(
        csharp_method_regex()
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    )
}

fn extract_declarations(source: &str) -> Vec<String> {
    sorted_unique(
        csharp_declaration_regex()
            .captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    )
}

fn extract_references(source: &str) -> Vec<String> {
    let keywords = csharp_reference_keywords();
    sorted_unique(
        csharp_reference_regex()
            .captures_iter(source)
            .filter_map(|cap| {
                let raw = cap.get(0)?.as_str();
                (!keywords.contains(raw)).then(|| raw.to_string())
            }),
    )
}

pub(super) fn csharp_reference_keywords() -> &'static HashSet<&'static str> {
    CSHARP_REFERENCE_KEYWORDS.get_or_init(|| {
        [
            "Console",
            "DateTime",
            "Exception",
            "False",
            "List",
            "Math",
            "Nullable",
            "Object",
            "String",
            "Task",
            "True",
            "ValueTask",
        ]
        .into_iter()
        .collect()
    })
}

fn has_xunit_tests(source: &str) -> bool {
    csharp_xunit_regex().is_match(source)
}

static CSHARP_FILE_NAMESPACE_REGEX: OnceLock<Regex> = OnceLock::new();
static CSHARP_BLOCK_NAMESPACE_REGEX: OnceLock<Regex> = OnceLock::new();
static CSHARP_USING_REGEX: OnceLock<Regex> = OnceLock::new();
static CSHARP_METHOD_REGEX: OnceLock<Regex> = OnceLock::new();
static CSHARP_DECLARATION_REGEX: OnceLock<Regex> = OnceLock::new();
static CSHARP_REFERENCE_REGEX: OnceLock<Regex> = OnceLock::new();
static CSHARP_XUNIT_REGEX: OnceLock<Regex> = OnceLock::new();
static CSHARP_REFERENCE_KEYWORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

pub(super) fn csharp_file_namespace_regex() -> &'static Regex {
    CSHARP_FILE_NAMESPACE_REGEX.get_or_init(|| {
        Regex::new(r"(?m)^\s*namespace\s+([A-Za-z_][A-Za-z0-9_.]*)\s*;").expect("valid regex")
    })
}

fn csharp_block_namespace_regex() -> &'static Regex {
    CSHARP_BLOCK_NAMESPACE_REGEX.get_or_init(|| {
        Regex::new(r"(?m)^\s*namespace\s+([A-Za-z_][A-Za-z0-9_.]*)\s*\{").expect("valid regex")
    })
}

fn csharp_using_regex() -> &'static Regex {
    CSHARP_USING_REGEX.get_or_init(|| {
        Regex::new(r"(?m)^\s*(?:global\s+)?using\s+(?:static\s+)?(?:[A-Za-z_][A-Za-z0-9_]*\s*=\s*)?([A-Za-z_][A-Za-z0-9_.]*)\s*;")
            .expect("valid regex")
    })
}

fn csharp_method_regex() -> &'static Regex {
    CSHARP_METHOD_REGEX.get_or_init(|| {
        Regex::new(r"(?:public|internal|private|protected)\s+(?:(?:static|async|virtual|override|new|partial|extern|unsafe|sealed|abstract)\s+)*[\w.<>,\[\]?]+\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")
            .expect("valid regex")
    })
}

pub(super) fn csharp_declaration_regex() -> &'static Regex {
    CSHARP_DECLARATION_REGEX.get_or_init(|| {
        Regex::new(r"\b(?:(?:public|internal|private|protected|sealed|abstract|static|partial|readonly|record)\s+)*(?:class|struct|interface|enum|record)\s+([A-Za-z_][A-Za-z0-9_]*)")
            .expect("valid regex")
    })
}

pub(super) fn csharp_reference_regex() -> &'static Regex {
    CSHARP_REFERENCE_REGEX
        .get_or_init(|| Regex::new(r"\b[A-Z][A-Za-z0-9_]*\b").expect("valid regex"))
}

fn csharp_xunit_regex() -> &'static Regex {
    CSHARP_XUNIT_REGEX
        .get_or_init(|| Regex::new(r"\[(?:Xunit\.)?(?:Fact|Theory)(?:\(|\])").expect("valid regex"))
}

fn sorted_unique<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    let mut out: Vec<String> = values.into_iter().collect();
    out.sort();
    out.dedup();
    out
}
