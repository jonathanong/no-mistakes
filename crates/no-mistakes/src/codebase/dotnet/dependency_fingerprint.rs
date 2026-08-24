use super::dependency_diff::parse_open_tag;
use super::DotnetDependencyDiagnostic;

#[derive(Debug, serde::Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct CanonicalElement {
    name: String,
    attributes: Vec<(String, String)>,
    text: String,
    children: Vec<CanonicalElement>,
}

pub(super) fn dependency_fingerprint(source: &str) -> Result<String, DotnetDependencyDiagnostic> {
    fingerprint(source, true)
}

/// Canonicalize a complete XML document while retaining element order, which
/// can be semantically meaningful to MSBuild. This is intentionally separate
/// from dependency item fingerprints, whose metadata children are a set.
pub(in crate::codebase::dotnet) fn full_document_fingerprint(
    source: &str,
) -> Result<String, DotnetDependencyDiagnostic> {
    fingerprint(source, false)
}

fn fingerprint(source: &str, sort_children: bool) -> Result<String, DotnetDependencyDiagnostic> {
    let mut roots = Vec::new();
    let mut stack = Vec::new();
    let mut cursor = 0;
    while cursor < source.len() {
        let Some(start) = source[cursor..].find('<').map(|offset| cursor + offset) else {
            if source[cursor..].trim().is_empty() {
                break;
            }
            return Err(DotnetDependencyDiagnostic::MalformedXml);
        };
        push_text(&mut stack, &source[cursor..start]);
        if source[start..].starts_with("<!--") {
            cursor = source[start + 4..]
                .find("-->")
                .map(|offset| start + offset + 7)
                .ok_or(DotnetDependencyDiagnostic::MalformedXml)?;
            continue;
        }
        if source[start..].starts_with("<?") {
            cursor = source[start + 2..]
                .find("?>")
                .map(|offset| start + offset + 4)
                .ok_or(DotnetDependencyDiagnostic::MalformedXml)?;
            continue;
        }
        if source[start..].starts_with("<![CDATA[") {
            let content_start = start + "<![CDATA[".len();
            let content_end = source[content_start..]
                .find("]]>")
                .map(|offset| content_start + offset)
                .ok_or(DotnetDependencyDiagnostic::MalformedXml)?;
            push_text(&mut stack, &source[content_start..content_end]);
            cursor = content_end + 3;
            continue;
        }
        let end = source[start..]
            .find('>')
            .map(|offset| start + offset + 1)
            .ok_or(DotnetDependencyDiagnostic::MalformedXml)?;
        let tag = &source[start + 1..end - 1];
        if let Some(name) = tag.strip_prefix('/') {
            close_element(&mut stack, &mut roots, name.trim(), sort_children)?;
        } else {
            let (name, mut attributes, self_closing) = parse_open_tag(tag)?;
            attributes.sort();
            let element = CanonicalElement {
                name,
                attributes,
                text: String::new(),
                children: Vec::new(),
            };
            if self_closing {
                append_element(&mut stack, &mut roots, element);
            } else {
                stack.push(element);
            }
        }
        cursor = end;
    }
    if !stack.is_empty() || roots.len() != 1 {
        return Err(DotnetDependencyDiagnostic::MalformedXml);
    }
    serde_json::to_string(&roots[0]).map_err(|_| DotnetDependencyDiagnostic::MalformedXml)
}

fn push_text(stack: &mut [CanonicalElement], text: &str) {
    let text = text.trim();
    if let Some(element) = stack.last_mut().filter(|_| !text.is_empty()) {
        element.text.push_str(text);
    }
}

fn close_element(
    stack: &mut Vec<CanonicalElement>,
    roots: &mut Vec<CanonicalElement>,
    name: &str,
    sort_children: bool,
) -> Result<(), DotnetDependencyDiagnostic> {
    let Some(mut element) = stack.pop() else {
        return Err(DotnetDependencyDiagnostic::MalformedXml);
    };
    if element.name != name {
        return Err(DotnetDependencyDiagnostic::MalformedXml);
    }
    if sort_children {
        element.children.sort();
    }
    append_element(stack, roots, element);
    Ok(())
}

fn append_element(
    stack: &mut [CanonicalElement],
    roots: &mut Vec<CanonicalElement>,
    element: CanonicalElement,
) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(element);
    } else {
        roots.push(element);
    }
}
