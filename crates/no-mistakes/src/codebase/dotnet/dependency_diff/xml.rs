use super::DotnetDependencyDiagnostic;
use regex::Regex;

mod fingerprints;
mod parser;
pub(in crate::codebase::dotnet) use fingerprints::dependency_map;
pub(in crate::codebase::dotnet) use parser::parse_open_tag;

#[derive(Debug)]
pub(super) struct DependencyItem {
    pub(super) name: String,
    source: String,
    pub(super) start: usize,
    pub(super) end: usize,
}
#[derive(Debug)]
struct OpenElement {
    name: String,
    start: usize,
    conditional: bool,
    dependency: Option<String>,
}

pub(super) fn dependency_items(
    source: &str,
    tags: &[&str],
) -> Result<Vec<DependencyItem>, DotnetDependencyDiagnostic> {
    let mut items = Vec::new();
    let mut stack = Vec::new();
    let mut cursor = 0;
    let mut root_seen = false;
    while cursor < source.len() {
        let Some(relative) = source[cursor..].find('<') else {
            if !source[cursor..].trim().is_empty() && stack.is_empty() {
                return Err(DotnetDependencyDiagnostic::MalformedXml);
            }
            break;
        };
        let start = cursor + relative;
        if !source[cursor..start].trim().is_empty() && stack.is_empty() {
            return Err(DotnetDependencyDiagnostic::MalformedXml);
        }
        if let Some(end) = special_end(source, start)? {
            cursor = end;
            continue;
        }
        let end = source[start..]
            .find('>')
            .map(|end| start + end + 1)
            .ok_or(DotnetDependencyDiagnostic::MalformedXml)?;
        let tag = &source[start + 1..end - 1];
        if let Some(name) = tag.strip_prefix('/') {
            close_item(&mut items, &mut stack, source, end, name.trim())?;
        } else if tag.starts_with('!') {
            return Err(DotnetDependencyDiagnostic::MalformedXml);
        } else {
            open_item(
                &mut items,
                &mut stack,
                tags,
                source,
                start,
                end,
                &mut root_seen,
            )?;
        }
        cursor = end;
    }
    if !root_seen || !stack.is_empty() {
        return Err(DotnetDependencyDiagnostic::MalformedXml);
    }
    Ok(items)
}

fn special_end(source: &str, start: usize) -> Result<Option<usize>, DotnetDependencyDiagnostic> {
    for (prefix, suffix) in [("<!--", "-->"), ("<?", "?>"), ("<![CDATA[", "]]>")] {
        if source[start..].starts_with(prefix) {
            return source[start + prefix.len()..]
                .find(suffix)
                .map(|end| Some(start + prefix.len() + suffix.len() + end))
                .ok_or(DotnetDependencyDiagnostic::MalformedXml);
        }
    }
    Ok(None)
}

fn close_item(
    items: &mut Vec<DependencyItem>,
    stack: &mut Vec<OpenElement>,
    source: &str,
    end: usize,
    name: &str,
) -> Result<(), DotnetDependencyDiagnostic> {
    if name.is_empty() || name.contains(char::is_whitespace) {
        return Err(DotnetDependencyDiagnostic::MalformedXml);
    }
    let open = stack
        .pop()
        .ok_or(DotnetDependencyDiagnostic::MalformedXml)?;
    if open.name != name {
        return Err(DotnetDependencyDiagnostic::MalformedXml);
    }
    if let Some(name) = open.dependency {
        items.push(DependencyItem {
            name,
            source: source[open.start..end].to_string(),
            start: open.start,
            end,
        });
    }
    Ok(())
}

fn open_item(
    items: &mut Vec<DependencyItem>,
    stack: &mut Vec<OpenElement>,
    tags: &[&str],
    source: &str,
    start: usize,
    end: usize,
    root_seen: &mut bool,
) -> Result<(), DotnetDependencyDiagnostic> {
    let (name, attributes, self_closing) = parse_open_tag(&source[start + 1..end - 1])?;
    if name.eq_ignore_ascii_case("Import") || source[start..end].contains("$(") {
        return Err(DotnetDependencyDiagnostic::UnsupportedDynamicDeclaration);
    }
    if !*root_seen {
        if name != "Project" {
            return Err(DotnetDependencyDiagnostic::MalformedXml);
        }
        *root_seen = true;
    }
    let inherited = stack.iter().any(|element| element.conditional);
    let conditional = attributes
        .iter()
        .any(|(attribute, _)| attribute.eq_ignore_ascii_case("Condition"));
    let dependency = tags
        .contains(&name.as_str())
        .then(|| dependency_name(&attributes))
        .transpose()?;
    if dependency.is_some() && (conditional || inherited) {
        return Err(DotnetDependencyDiagnostic::UnsupportedDynamicDeclaration);
    }
    if dependency
        .as_ref()
        .is_some_and(|name| name.contains(['*', ';']) || name.contains("@(") || name.contains("%("))
    {
        return Err(DotnetDependencyDiagnostic::UnsupportedDynamicDeclaration);
    }
    if self_closing {
        if let Some(name) = dependency {
            items.push(DependencyItem {
                name,
                source: source[start..end].to_string(),
                start,
                end,
            });
        }
    } else {
        stack.push(OpenElement {
            name,
            start,
            conditional: conditional || inherited,
            dependency,
        });
    }
    Ok(())
}

fn dependency_name(attributes: &[(String, String)]) -> Result<String, DotnetDependencyDiagnostic> {
    attributes
        .iter()
        .find(|(attribute, _)| {
            attribute.eq_ignore_ascii_case("Include") || attribute.eq_ignore_ascii_case("Update")
        })
        .map(|(_, value)| value.clone())
        .ok_or(DotnetDependencyDiagnostic::UnsupportedDynamicDeclaration)
}

pub(super) fn validate_xml(source: &str) -> Result<(), DotnetDependencyDiagnostic> {
    if source.contains("$(")
        || source.contains("@(")
        || source.contains("%(")
        || source.contains("<Import")
    {
        return Err(DotnetDependencyDiagnostic::UnsupportedDynamicDeclaration);
    }
    dependency_items(source, &[]).map(|_| ())
}
pub(super) fn normalize_xml(source: &str) -> String {
    Regex::new(r">\s+<")
        .expect("valid XML trivia expression")
        .replace_all(source, "><")
        .into_owned()
}
