use super::super::{ResolutionKind, ResolvedPackage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GemfileError;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    Specs(ResolutionKind),
    Checksums,
    Other,
}

pub(crate) fn parse(content: &str) -> Vec<ResolvedPackage> {
    parse_checked(content).unwrap_or_default()
}

pub(crate) fn parse_checked(content: &str) -> Result<Vec<ResolvedPackage>, GemfileError> {
    let mut out: Vec<ResolvedPackage> = Vec::new();
    let mut section = Section::None;
    let mut saw_header = false;
    let mut revision = String::new();
    for raw in content.lines() {
        let line = raw.trim_end_matches('\r');
        let indent = line.chars().take_while(|ch| *ch == ' ').count();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if indent == 0 && is_header(trimmed) {
            saw_header = true;
            revision.clear();
            section = match trimmed {
                "GEM" => Section::Specs(ResolutionKind::Registry),
                "GIT" => Section::Specs(ResolutionKind::Git),
                "PATH" => Section::Specs(ResolutionKind::Directory),
                "CHECKSUMS" => Section::Checksums,
                _ => Section::Other,
            };
            continue;
        }
        if let Section::Specs(ResolutionKind::Git) = section {
            if let Some(rev) = trimmed.strip_prefix("revision:") {
                revision = rev.trim().to_string();
            }
        }
        if section == Section::Checksums {
            if let Some((name, version, Some(digest))) = spec(trimmed) {
                let Some(pkg) = out
                    .iter_mut()
                    .find(|pkg| pkg.name == name && pkg.version == version)
                else {
                    continue;
                };
                pkg.fingerprint = digest.to_string();
            }
            continue;
        }
        let Section::Specs(kind) = section else {
            continue;
        };
        if indent != 4 {
            continue;
        }
        let Some((name, version, _)) = spec(trimmed) else {
            return Err(GemfileError);
        };
        let fingerprint = if kind == ResolutionKind::Git {
            revision.clone()
        } else {
            String::new()
        };
        out.push(ResolvedPackage {
            name: name.to_string(),
            version: version.to_string(),
            fingerprint,
            kind,
        });
    }
    if !saw_header {
        return Err(GemfileError);
    }
    Ok(out)
}

fn is_header(line: &str) -> bool {
    line.chars().all(|ch| ch.is_ascii_uppercase() || ch == ' ')
}

fn spec(trimmed: &str) -> Option<(&str, &str, Option<&str>)> {
    let open = trimmed.find(" (")?;
    let name = trimmed[..open].trim();
    if name.is_empty() || name.contains(' ') {
        return None;
    }
    let after = &trimmed[open + 2..];
    let close = after.find(')')?;
    let version = &after[..close];
    let extra = after[close + 1..].trim();
    let digest = extra.strip_prefix("sha256=");
    Some((name, version, digest.filter(|value| !value.is_empty())))
}
