use super::super::{ResolutionKind, ResolvedPackage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GoModError;

pub(crate) fn parse(content: &str) -> Vec<ResolvedPackage> {
    parse_checked(content).unwrap_or_default()
}

pub(crate) fn parse_checked(content: &str) -> Result<Vec<ResolvedPackage>, GoModError> {
    let mut out = Vec::new();
    let mut in_require = false;
    let mut saw_module = false;
    for raw in content.lines() {
        let line = strip_comment(raw).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("module ") || line.starts_with("go ") {
            saw_module = true;
        }
        if line == "require (" {
            in_require = true;
            continue;
        }
        if in_require && line == ")" {
            in_require = false;
            continue;
        }
        if let Some(rest) = line.strip_prefix("require ") {
            if rest == "(" {
                in_require = true;
            } else {
                push_require(rest, &mut out)?;
            }
            continue;
        }
        if in_require {
            push_require(&line, &mut out)?;
        }
    }
    if in_require || (out.is_empty() && !saw_module) {
        return Err(GoModError);
    }
    Ok(out)
}

fn push_require(rest: &str, out: &mut Vec<ResolvedPackage>) -> Result<(), GoModError> {
    let mut parts = rest.split_whitespace();
    let name = parts.next().ok_or(GoModError)?;
    let version = parts.next().ok_or(GoModError)?;
    if !version.starts_with('v') {
        return Err(GoModError);
    }
    out.push(ResolvedPackage {
        name: name.to_string(),
        version: version.to_string(),
        fingerprint: String::new(),
        kind: ResolutionKind::Registry,
    });
    Ok(())
}

fn strip_comment(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}
