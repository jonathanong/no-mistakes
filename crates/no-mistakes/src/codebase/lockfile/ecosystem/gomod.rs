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
        let line = strip_comment(raw).trim();
        if in_require {
            if line == ")" {
                in_require = false;
            } else if !line.is_empty() {
                push_require(line, &mut out)?;
            }
            continue;
        }
        let end = line.find(|ch: char| ch.is_whitespace() || ch == '(');
        let (directive, rest) = line.split_at(end.unwrap_or(line.len()));
        match directive {
            "module" | "go" if !rest.trim().is_empty() => saw_module = true,
            "require" if rest.trim() == "(" => in_require = true,
            "require" => push_require(rest, &mut out)?,
            _ => {}
        }
    }
    if in_require || (out.is_empty() && !saw_module) {
        return Err(GoModError);
    }
    Ok(out)
}

fn push_require(rest: &str, out: &mut Vec<ResolvedPackage>) -> Result<(), GoModError> {
    let mut parts = rest.split_whitespace();
    let name = token(parts.next().ok_or(GoModError)?)?;
    let version = token(parts.next().ok_or(GoModError)?)?;
    if name.is_empty() || !version.starts_with('v') || parts.next().is_some() {
        return Err(GoModError);
    }
    out.push(ResolvedPackage {
        name,
        version,
        fingerprint: String::new(),
        kind: ResolutionKind::Registry,
    });
    Ok(())
}

fn token(raw: &str) -> Result<String, GoModError> {
    if raw.starts_with('"') {
        return serde_json::from_str(raw).map_err(|_| GoModError);
    }
    if let Some(rest) = raw.strip_prefix('`') {
        return rest
            .strip_suffix('`')
            .filter(|body| !body.contains('`'))
            .map(str::to_string)
            .ok_or(GoModError);
    }
    Ok(raw.to_string())
}

fn strip_comment(line: &str) -> &str {
    line.split_once("//").map_or(line, |(code, _)| code)
}
