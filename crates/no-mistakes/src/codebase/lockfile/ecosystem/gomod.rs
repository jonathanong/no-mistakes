use super::super::{ResolutionKind, ResolvedPackage};
use regex::Regex;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GoModError;

enum Block {
    Module(bool),
    Requirements,
    Replacements,
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
                saw_module |= matches!(active, Block::Module(true));
                block = None;
            } else {
                read_block_entry(active, line, &mut out)?;
            }
        } else {
            block = read_directive(line, &mut saw_module, &mut out)?;
        }
    }
    if block.is_some() || !saw_module {
        return Err(GoModError);
    }
    Ok(out)
}

fn read_block_entry(
    block: &mut Block,
    line: &str,
    out: &mut Vec<ResolvedPackage>,
) -> Result<(), GoModError> {
    match block {
        Block::Module(seen) if !*seen && has_one_token(line) => *seen = true,
        Block::Module(_) => return Err(GoModError),
        Block::Requirements => push_require(line, out)?,
        Block::Replacements if !replacement_fields(line) => return Err(GoModError),
        Block::Replacements | Block::Other => {}
    }
    Ok(())
}

fn read_directive(
    line: &str,
    saw_module: &mut bool,
    out: &mut Vec<ResolvedPackage>,
) -> Result<Option<Block>, GoModError> {
    let end = line.find(|ch: char| ch.is_whitespace() || ch == '(');
    let (directive, rest) = line.split_at(end.unwrap_or(line.len()));
    let rest = rest.trim();
    match directive {
        "module" if !*saw_module && rest == "(" => return Ok(Some(Block::Module(false))),
        "module" if !*saw_module && has_one_token(rest) => *saw_module = true,
        "go" | "toolchain" if has_one_token(rest) => {}
        "require" | "replace" if inline_empty_block(rest) => {}
        "require" if rest == "(" => return Ok(Some(Block::Requirements)),
        "require" => push_require(rest, out)?,
        "replace" if rest == "(" => return Ok(Some(Block::Replacements)),
        "replace" if replacement_fields(rest) => {}
        "exclude" | "retract" | "godebug" | "tool" | "ignore" if rest == "(" => {
            return Ok(Some(Block::Other));
        }
        "exclude" | "retract" | "godebug" | "tool" | "ignore" if !rest.is_empty() => {}
        _ => return Err(GoModError),
    }
    Ok(None)
}

fn inline_empty_block(rest: &str) -> bool {
    rest.strip_prefix('(')
        .and_then(|rest| rest.strip_suffix(')'))
        .is_some_and(|inside| inside.trim().is_empty())
}

fn replacement_fields(line: &str) -> bool {
    static REPLACEMENT: OnceLock<Regex> = OnceLock::new();
    REPLACEMENT
        .get_or_init(|| {
            let atom = r#""(?:[^"\\]|\\.)*"|`[^`]*`|[^\s"`()\[\]{},]+"#;
            let operands = format!(r"({atom})(?:\s+({atom}))?");
            Regex::new(&format!(r"^{operands}\s+=>\s+{operands}$")).expect("Go module replacement")
        })
        .captures(line)
        .is_some_and(|parts| {
            parts
                .iter()
                .skip(1)
                .flatten()
                .all(|part| part.as_str() != "=>")
        })
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
    let bytes = line.as_bytes();
    let mut index = 0;
    let mut quoted = None;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(ender) = quoted {
            if ender == b'"' && byte == b'\\' {
                index = (index + 2).min(bytes.len());
                continue;
            }
            if byte == ender {
                quoted = None;
            }
            index += 1;
            continue;
        }
        if matches!(byte, b'"' | b'`') {
            quoted = Some(byte);
        } else if byte == b'/' && bytes.get(index + 1) == Some(&b'/') {
            return &line[..index];
        }
        index += 1;
    }
    line
}
