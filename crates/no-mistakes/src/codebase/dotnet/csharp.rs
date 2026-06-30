use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::{csharp_strip::strip_comments_and_strings, DotnetFileFacts};

pub(crate) fn parse_csharp_file(path: &Path, project: Option<PathBuf>) -> Option<DotnetFileFacts> {
    let source = std::fs::read_to_string(path).ok()?;
    let stripped = strip_comments_and_strings(&source);
    Some(DotnetFileFacts {
        path: path.to_path_buf(),
        project,
        namespace: extract_namespace(&stripped),
        usings: extract_usings(&stripped),
        declarations: extract_declarations(&stripped),
        references: extract_references(&stripped),
        has_xunit_tests: has_xunit_tests(&stripped),
    })
}

fn extract_namespace(source: &str) -> Option<String> {
    let file_scoped =
        Regex::new(r"(?m)^\s*namespace\s+([A-Za-z_][A-Za-z0-9_.]*)\s*;").expect("valid regex");
    let block =
        Regex::new(r"(?m)^\s*namespace\s+([A-Za-z_][A-Za-z0-9_.]*)\s*\{").expect("valid regex");
    file_scoped
        .captures(source)
        .or_else(|| block.captures(source))
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

fn extract_usings(source: &str) -> Vec<String> {
    let re = Regex::new(
        r"(?m)^\s*(?:global\s+)?using\s+(?:static\s+)?(?:[A-Za-z_][A-Za-z0-9_]*\s*=\s*)?([A-Za-z_][A-Za-z0-9_.]*)\s*;",
    )
    .expect("valid regex");
    sorted_unique(
        re.captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    )
}

fn extract_declarations(source: &str) -> Vec<String> {
    let re = Regex::new(
        r"\b(?:(?:public|internal|private|protected|sealed|abstract|static|partial|readonly|record)\s+)*(?:class|struct|interface|enum|record)\s+([A-Za-z_][A-Za-z0-9_]*)",
    )
    .expect("valid regex");
    sorted_unique(
        re.captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string())),
    )
}

fn extract_references(source: &str) -> Vec<String> {
    let re = Regex::new(r"\b[A-Z][A-Za-z0-9_]*\b").expect("valid regex");
    let keywords = csharp_reference_keywords();
    sorted_unique(re.captures_iter(source).filter_map(|cap| {
        let raw = cap.get(0)?.as_str();
        (!keywords.contains(raw)).then(|| raw.to_string())
    }))
}

fn csharp_reference_keywords() -> HashSet<&'static str> {
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
}

fn has_xunit_tests(source: &str) -> bool {
    Regex::new(r"\[(?:Xunit\.)?(?:Fact|Theory)(?:\(|\])")
        .expect("valid regex")
        .is_match(source)
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
