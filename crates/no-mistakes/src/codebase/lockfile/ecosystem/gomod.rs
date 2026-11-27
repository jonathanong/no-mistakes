use super::super::{ResolutionKind, ResolvedPackage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GoModError;

enum Block {
    Module(bool),
    Requirements,
    Other,
}

pub(crate) fn parse(content: &str) -> Vec<ResolvedPackage> {
    parse_checked(content).unwrap_or_default()
}

pub(crate) fn parse_checked(content: &str) -> Result<Vec<ResolvedPackage>, GoModError> {
    let mut out = Vec::new();
    let mut block = None;
    let mut saw_module = false;
    for raw in content.lines() {
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if ["<<<<<<<", "|||||||", "=======", ">>>>>>>"]
            .iter()
            .any(|marker| line.starts_with(marker))
        {
            return Err(GoModError);
        }
        if let Some(active) = block.as_mut() {
            if line == ")" {
                if matches!(active, Block::Module(false)) {
                    return Err(GoModError);
                }
                block = None;
            } else {
                match active {
                    Block::Module(seen) if !*seen && has_one_token(line) => {
                        *seen = true;
                        saw_module = true;
                    }
                    Block::Module(_) => return Err(GoModError),
                    Block::Requirements => push_require(line, &mut out)?,
                    Block::Other => {}
                }
            }
            continue;
        }
        let end = line.find(|ch: char| ch.is_whitespace() || ch == '(');
        let (directive, rest) = line.split_at(end.unwrap_or(line.len()));
        let rest = rest.trim();
        match directive {
            "module" if rest == "(" => block = Some(Block::Module(false)),
            "module" if has_one_token(rest) => saw_module = true,
            "go" if has_one_token(rest) => {}
            "toolchain" if has_one_token(rest) => {}
            "require" if rest == "(" => block = Some(Block::Requirements),
            "require" => push_require(rest, &mut out)?,
            "replace" | "exclude" | "retract" | "godebug" | "tool" | "ignore" if rest == "(" => {
                block = Some(Block::Other);
            }
            "replace" | "exclude" | "retract" | "godebug" | "tool" | "ignore"
                if !rest.is_empty() => {}
            _ => return Err(GoModError),
        }
    }
    if block.is_some() || !saw_module {
        return Err(GoModError);
    }
    Ok(out)
}

fn has_one_token(rest: &str) -> bool {
    let mut tokens = rest.split_whitespace();
    tokens
        .next()
        .is_some_and(|token| !matches!(token, "(" | ")"))
        && tokens.next().is_none()
}

fn push_require(rest: &str, out: &mut Vec<ResolvedPackage>) -> Result<(), GoModError> {
    let mut parts = rest.split_whitespace();
    let name = token(parts.next().ok_or(GoModError)?)?;
    let version = token(parts.next().ok_or(GoModError)?)?;
    if name.is_empty()
        || version.is_empty()
        || matches!(version.as_str(), "(" | ")" | "=>")
        || parts.next().is_some()
    {
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
